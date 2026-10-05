import uuid
import pytest
from httpx import AsyncClient, ASGITransport
from app.main import app

@pytest.mark.asyncio
async def test_auth_registration_and_duplicate():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        uid = uuid.uuid4().hex[:8]
        # Register new user
        reg_payload = {
            "email": f"trader_{uid}@quant.local",
            "username": f"trader_{uid}",
            "password": "SecurePassword123!",
            "full_name": "Trader One",
        }
        res = await client.post("/api/v1/auth/register", json=reg_payload)
        assert res.status_code == 201
        data = res.json()
        assert data["username"] == f"trader_{uid}"
        assert data["email"] == f"trader_{uid}@quant.local"
        assert "hashed_password" not in data

        # Duplicate registration should be rejected
        res_dup = await client.post("/api/v1/auth/register", json=reg_payload)
        assert res_dup.status_code == 400

@pytest.mark.asyncio
async def test_multi_login_concurrent_sessions():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. Login from Device A: MacBook Pro
        login_a = {
            "username_or_email": "trader1",
            "password": "SecurePassword123!",
            "device_name": "MacBook Pro M3",
        }
        res_a = await client.post("/api/v1/auth/login", json=login_a)
        assert res_a.status_code == 200
        data_a = res_a.json()
        token_a = data_a["access_token"]
        session_a = data_a["session_id"]
        refresh_a = data_a["refresh_token"]

        # 2. Login from Device B: iPhone Mobile App (concurrent session)
        login_b = {
            "username_or_email": "trader1@quant.local",
            "password": "SecurePassword123!",
            "device_name": "iPhone 16 iOS",
        }
        res_b = await client.post("/api/v1/auth/login", json=login_b)
        assert res_b.status_code == 200
        data_b = res_b.json()
        token_b = data_b["access_token"]
        session_b = data_b["session_id"]

        # 3. Login from Device C: MetaTrader 5 VPS
        login_c = {
            "username_or_email": "trader1",
            "password": "SecurePassword123!",
            "device_name": "MT5 Cloud VPS",
        }
        res_c = await client.post("/api/v1/auth/login", json=login_c)
        assert res_c.status_code == 200
        data_c = res_c.json()
        token_c = data_c["access_token"]
        session_c = data_c["session_id"]

        # Verify that all 3 tokens are distinct sessions
        assert len({session_a, session_b, session_c}) == 3

        # Verify that ALL 3 sessions are concurrently active and can access protected /auth/me
        for tok in [token_a, token_b, token_c]:
            headers = {"Authorization": f"Bearer {tok}"}
            me_res = await client.get("/api/v1/auth/me", headers=headers)
            assert me_res.status_code == 200
            assert me_res.json()["username"] == "trader1"

        # 4. Inspect active sessions list
        headers_a = {"Authorization": f"Bearer {token_a}"}
        sess_list_res = await client.get("/api/v1/auth/sessions", headers=headers_a)
        assert sess_list_res.status_code == 200
        sessions = sess_list_res.json()
        assert len(sessions) >= 3
        device_names = [s["device_name"] for s in sessions]
        assert "MacBook Pro M3" in device_names
        assert "iPhone 16 iOS" in device_names
        assert "MT5 Cloud VPS" in device_names

        # Session A should report is_current=True when requested with token_a
        current_sess = next(s for s in sessions if s["session_id"] == session_a)
        assert current_sess["is_current"] is True

        # 5. Targeted session logout: revoke session B (iPhone)
        logout_b_res = await client.post(f"/api/v1/auth/logout?session_id={session_b}", headers=headers_a)
        assert logout_b_res.status_code == 200

        # Token B should now be rejected immediately!
        headers_b = {"Authorization": f"Bearer {token_b}"}
        revoked_check = await client.get("/api/v1/auth/me", headers=headers_b)
        assert revoked_check.status_code == 401

        # Meanwhile, Token A and Token C MUST STILL BE FULLY OPERATIONAL!
        headers_c = {"Authorization": f"Bearer {token_c}"}
        assert (await client.get("/api/v1/auth/me", headers=headers_a)).status_code == 200
        assert (await client.get("/api/v1/auth/me", headers=headers_c)).status_code == 200

        # 6. Test Refresh Token flow on Session A
        refresh_res = await client.post("/api/v1/auth/refresh", json={"refresh_token": refresh_a})
        assert refresh_res.status_code == 200
        new_data_a = refresh_res.json()
        assert new_data_a["session_id"] == session_a
        new_token_a = new_data_a["access_token"]
        assert (await client.get("/api/v1/auth/me", headers={"Authorization": f"Bearer {new_token_a}"})).status_code == 200

        # 7. Logout all remaining sessions
        logout_all_res = await client.post("/api/v1/auth/logout-all", headers=headers_c)
        assert logout_all_res.status_code == 200

        # Now all previous tokens should be rejected
        assert (await client.get("/api/v1/auth/me", headers={"Authorization": f"Bearer {new_token_a}"})).status_code == 401
        assert (await client.get("/api/v1/auth/me", headers=headers_c)).status_code == 401

@pytest.mark.asyncio
async def test_bot_api_key_authentication():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # Login to generate an API key
        login_res = await client.post("/api/v1/auth/login", json={
            "username_or_email": "trader1",
            "password": "SecurePassword123!",
            "device_name": "API Key Generator",
        })
        token = login_res.json()["access_token"]
        headers = {"Authorization": f"Bearer {token}"}

        # Create API Key for trading bot
        key_res = await client.post("/api/v1/auth/api-keys", json={
            "name": "Production MT5 Ingestor Bot",
            "expires_days": 90,
        }, headers=headers)
        assert key_res.status_code == 201
        key_data = key_res.json()
        api_key = key_data["api_key"]
        key_id = key_data["id"]
        assert api_key.startswith("tk_")

        # Access /api/v1/auth/me using ONLY the X-API-Key header
        bot_headers = {"X-API-Key": api_key}
        bot_me_res = await client.get("/api/v1/auth/me", headers=bot_headers)
        assert bot_me_res.status_code == 200
        assert bot_me_res.json()["username"] == "trader1"

        # Tokenize with attribution via API key
        tok_res = await client.post("/api/v1/tokens/tokenize", json={
            "text": "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800",
            "persist": True,
        }, headers=bot_headers)
        assert tok_res.status_code == 200
        assert tok_res.json()["record_id"] is not None

        # Revoke API key
        del_res = await client.delete(f"/api/v1/auth/api-keys/{key_id}", headers=headers)
        assert del_res.status_code == 200

        # After revocation, API key is rejected
        rejected_res = await client.get("/api/v1/auth/me", headers=bot_headers)
        assert rejected_res.status_code == 401

@pytest.mark.asyncio
async def test_multi_user_data_isolation():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # Register User Alpha
        await client.post("/api/v1/auth/register", json={
            "email": "alpha@quant.local",
            "username": "alpha_trader",
            "password": "PasswordAlpha123!",
        })
        res_a = await client.post("/api/v1/auth/login", json={
            "username_or_email": "alpha_trader",
            "password": "PasswordAlpha123!",
        })
        token_a = res_a.json()["access_token"]
        headers_a = {"Authorization": f"Bearer {token_a}"}

        # Register User Beta
        await client.post("/api/v1/auth/register", json={
            "email": "beta@quant.local",
            "username": "beta_trader",
            "password": "PasswordBeta123!",
        })
        res_b = await client.post("/api/v1/auth/login", json={
            "username_or_email": "beta_trader",
            "password": "PasswordBeta123!",
        })
        token_b = res_b.json()["access_token"]
        headers_b = {"Authorization": f"Bearer {token_b}"}

        # User Alpha creates a private template
        tpl_a_res = await client.post("/api/v1/templates", json={
            "name": "Alpha Private Forex Pattern",
            "pattern_syntax": "ALPHA $(symbol) $(action) $(entry_price)",
            "template_type": "market",
        }, headers=headers_a)
        assert tpl_a_res.status_code == 201
        tpl_a_id = tpl_a_res.json()["id"]

        # User Beta creates a private template
        tpl_b_res = await client.post("/api/v1/templates", json={
            "name": "Beta Private Crypto Pattern",
            "pattern_syntax": "BETA $(symbol) $(action) $(entry_price)",
            "template_type": "market",
        }, headers=headers_b)
        assert tpl_b_res.status_code == 201
        tpl_b_id = tpl_b_res.json()["id"]

        # 1. User Alpha lists templates: sees Alpha template, does NOT see Beta template
        list_a = (await client.get("/api/v1/templates", headers=headers_a)).json()
        ids_a = [t["id"] for t in list_a]
        assert tpl_a_id in ids_a
        assert tpl_b_id not in ids_a

        # 2. User Beta lists templates: sees Beta template, does NOT see Alpha template
        list_b = (await client.get("/api/v1/templates", headers=headers_b)).json()
        ids_b = [t["id"] for t in list_b]
        assert tpl_b_id in ids_b
        assert tpl_a_id not in ids_b

        # 3. User Beta cannot access Alpha's template directly
        get_b_of_a = await client.get(f"/api/v1/templates/{tpl_a_id}", headers=headers_b)
        assert get_b_of_a.status_code == 403

        # 4. User Beta cannot update Alpha's template
        upd_b_of_a = await client.put(f"/api/v1/templates/{tpl_a_id}", json={"name": "Hacked"}, headers=headers_b)
        assert upd_b_of_a.status_code == 403

        # 5. User Beta cannot delete Alpha's template
        del_b_of_a = await client.delete(f"/api/v1/templates/{tpl_a_id}", headers=headers_b)
        assert del_b_of_a.status_code == 403

        # 6. User Alpha can successfully delete their own template
        del_a = await client.delete(f"/api/v1/templates/{tpl_a_id}", headers=headers_a)
        assert del_a.status_code == 200
