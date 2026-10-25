import pytest
from httpx import AsyncClient, ASGITransport
from app.main import app

@pytest.mark.asyncio
async def test_validator_stats():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/api/v1/validator/stats")
        assert res.status_code == 200
        data = res.json()
        assert "total_messages" in data
        assert "total_templates" in data
        assert "total_validated" in data
        assert data["total_messages"] > 4000
        assert data["total_templates"] >= 50

@pytest.mark.asyncio
async def test_validator_messages():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # Default query: excludes WhatsApp senders (wa*)
        res = await client.get("/api/v1/validator/messages?page=1&page_size=10")
        assert res.status_code == 200
        data = res.json()
        assert len(data["items"]) == 10
        assert data["page"] == 1
        assert data["total"] >= 3700
        # Verify no WhatsApp senders in results
        for it in data["items"]:
            if it["sender_name"]:
                assert not it["sender_name"].lower().startswith("wa")
        first = data["items"][0]
        assert "raw_text" in first
        assert "sender_name" in first

        # Query with include_wa=true includes all 4200+ messages
        res_all = await client.get("/api/v1/validator/messages?page=1&page_size=10&include_wa=true")
        assert res_all.status_code == 200
        assert res_all.json()["total"] > 4000

@pytest.mark.asyncio
async def test_validator_senders_excludes_whatsapp():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/api/v1/validator/senders")
        assert res.status_code == 200
        senders = res.json()
        assert len(senders) >= 40
        for s in senders:
            assert not s["display_name"].lower().startswith("wa")

@pytest.mark.asyncio
async def test_validator_test_exact():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        payload = {
            "text": "Gold buy now @4102\n\nSL:4001\n\nTp1-4106",
            "pattern_syntax": "$(symbol) $(action) now @$(entry_price)\n\nSL:$(stoploss)\n\nTp$(index)-$(profit_price)"
        }
        res = await client.post("/api/v1/validator/test-exact", json=payload)
        assert res.status_code == 200
        data = res.json()
        assert data["is_match"] is True
        assert data["signals_count"] == 1
        assert "signals" in data["signals_json"]
        assert data["latency_us"] > 0

@pytest.mark.asyncio
async def test_validator_test_batch():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        payload = {
            "pattern_syntax": "🕯 #$(symbol) $(action) $(entry_price)\n\nTP $(profit_price)\n\n🔴 SL_$(stoploss)",
            "mode": "at_least_one",
        }
        res = await client.post("/api/v1/validator/test-batch", json=payload)
        assert res.status_code == 200
        data = res.json()
        assert data["total_tested"] > 0
        assert "matched_count" in data
        assert "signals_total_count" in data
        assert "summary_by_sender" in data

@pytest.mark.asyncio
async def test_validator_parse():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        payload = {
            "text": "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800"
        }
        res = await client.post("/api/v1/validator/parse", json=payload)
        assert res.status_code == 200
        data = res.json()
        assert data["parse_signal_json"] is not None
        assert data["detected_signal_count"] >= 1
        assert "parse_signal_us" in data
        assert "signals" in data["parse_signal_json"]

@pytest.mark.asyncio
async def test_validator_validate_and_export(tmp_path, monkeypatch):
    import shutil
    from app.core.config import settings
    from app.api.v1.endpoints import validator

    orig_db = validator.get_db_path()
    temp_db = tmp_path / "consolidated_signals.db"
    shutil.copy2(orig_db, temp_db)
    monkeypatch.setattr(settings, "CONSOLIDATED_DB_PATH", temp_db)
    monkeypatch.setattr(settings, "DATA_DIR", tmp_path)

    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # Validate message #1
        val_payload = {
            "message_id": 1,
            "is_signal": True,
            "signals_count": 1,
            "ground_truth_json": '[{"symbol": "GBPJPY-OTC", "action": "BUY"}]',
            "template_name": "Test Template",
            "notes": "Automated test validation"
        }
        res_val = await client.post("/api/v1/validator/validate", json=val_payload)
        assert res_val.status_code == 200
        assert res_val.json()["status"] == "success"

        # Verify detail
        res_detail = await client.get("/api/v1/validator/messages/1")
        assert res_detail.status_code == 200
        detail = res_detail.json()
        assert detail["is_validated"] is True
        assert detail["is_signal"] is True
        assert detail["signals_count"] == 1
        assert detail["ground_truth_json"] == val_payload["ground_truth_json"]

        # Export test suite
        res_exp = await client.get("/api/v1/validator/export-tests?format=suite")
        assert res_exp.status_code == 200
        exp_data = res_exp.json()
        assert "cases" in exp_data
        assert "metadata" in exp_data
        assert exp_data["metadata"]["total_cases"] >= 1

        # Export signals_json format
        res_exp_sig = await client.get("/api/v1/validator/export-tests?format=signals_json")
        assert res_exp_sig.status_code == 200
        assert "dataCases" in res_exp_sig.json()

@pytest.mark.asyncio
async def test_validator_ui_route():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/validator")
        assert res.status_code == 200
        assert "Trading Signals Ground Truth Studio" in res.text
        assert "vue.global" in res.text
        assert 'list="token-tags-datalist"' in res.text
        assert 'v-model="syncScroll"' in res.text
        assert '@scroll="onTemplateScroll"' in res.text
        assert '@scroll="onRawTextScroll"' in res.text
        assert 'headerMatchStatus' in res.text
        assert 'showSignalsSummary' in res.text
        assert 'Resumen Estadístico de las Señales' in res.text

@pytest.mark.asyncio
async def test_templates_filter_by_sender_and_crud(tmp_path, monkeypatch):
    import shutil
    from app.core.config import settings
    from app.api.v1.endpoints import validator

    orig_db = validator.get_db_path()
    temp_db = tmp_path / "consolidated_signals.db"
    shutil.copy2(orig_db, temp_db)
    monkeypatch.setattr(settings, "CONSOLIDATED_DB_PATH", temp_db)
    monkeypatch.setattr(settings, "DATA_DIR", tmp_path)

    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. Fetch all templates
        res_all = await client.get("/api/v1/validator/templates")
        assert res_all.status_code == 200
        all_tpls = res_all.json()
        assert len(all_tpls) >= 40

        # 2. Filter by specific sender
        target_sender = "Forex treding"
        res_filtered = await client.get(f"/api/v1/validator/templates?sender_name={target_sender}")
        assert res_filtered.status_code == 200
        filtered_tpls = res_filtered.json()
        assert len(filtered_tpls) >= 1
        for t in filtered_tpls:
            assert t["sender_name"] == target_sender

        # 3. Create a new template linked to this sender
        new_payload = {
            "name": "Custom Scalp Forex Treding",
            "pattern_syntax": "🕯 #$(symbol) $(action) $(entry_price)\nTP $(profit_price)\nSL $(stoploss)",
            "sender_name": target_sender,
            "template_type": "market",
            "priority": 150
        }
        res_create = await client.post("/api/v1/validator/templates", json=new_payload)
        assert res_create.status_code == 201
        created = res_create.json()
        assert created["name"] == new_payload["name"]
        assert created["sender_name"] == target_sender
        created_id = created["id"]

        # 4. Verify the newly created template is returned in the sender's filtered list
        res_recheck = await client.get(f"/api/v1/validator/templates?sender_name={target_sender}")
        assert res_recheck.status_code == 200
        recheck_ids = [t["id"] for t in res_recheck.json()]
        assert created_id in recheck_ids

        # 5. Update the template (U in CRUD)
        update_payload = {
            "name": "Updated Scalp Forex Treding V2",
            "pattern_syntax": "🕯 #$(symbol) $(action) $(entry_price)\nTP$(index) $(profit_price)\nSL $(stoploss)",
            "priority": 180
        }
        res_upd = await client.put(f"/api/v1/validator/templates/{created_id}", json=update_payload)
        assert res_upd.status_code == 200
        updated = res_upd.json()
        assert updated["id"] == created_id
        assert updated["name"] == update_payload["name"]
        assert updated["pattern_syntax"] == update_payload["pattern_syntax"]
        assert updated["priority"] == 180
        assert updated["sender_name"] == target_sender

        # 6. Delete the template (D in CRUD)
        res_del = await client.delete(f"/api/v1/validator/templates/{created_id}")
        assert res_del.status_code == 200
        assert res_del.json()["status"] == "success"

        # 7. Verify deletion
        res_final = await client.get(f"/api/v1/validator/templates?sender_name={target_sender}")
        assert res_final.status_code == 200
        final_ids = [t["id"] for t in res_final.json()]
        assert created_id not in final_ids


@pytest.mark.asyncio
async def test_parse_iterates_all_templates_and_includes_newly_created(tmp_path, monkeypatch):
    import shutil
    from app.core.config import settings
    from app.api.v1.endpoints import validator

    orig_db = validator.get_db_path()
    temp_db = tmp_path / "consolidated_signals.db"
    shutil.copy2(orig_db, temp_db)
    monkeypatch.setattr(settings, "CONSOLIDATED_DB_PATH", temp_db)
    monkeypatch.setattr(settings, "DATA_DIR", tmp_path)

    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. Get total templates count
        res_all = await client.get("/api/v1/validator/templates")
        initial_count = len(res_all.json())

        # 2. Test a signal that does not match any existing template yet
        custom_signal_text = "🚀 SUPER VIP SIGNAL 🚀\nPAR: EURUSD\nACCION: BUY\nPRECIO: 1.0855\nTP1: 1.0910\nSL: 1.0810"
        res_parse_before = await client.post(
            "/api/v1/validator/parse",
            json={"text": custom_signal_text, "sender_name": "Canal VIP Test"},
        )
        assert res_parse_before.status_code == 200
        before_data = res_parse_before.json()
        assert before_data["templates_tested_count"] == initial_count
        assert before_data["exact_match"] is False

        # 3. Create a new template for this signal format
        new_tpl = {
            "name": "Plantilla Super VIP",
            "sender_name": "Canal VIP Test",
            "pattern_syntax": "🚀 SUPER VIP SIGNAL 🚀\nPAR: $(symbol)\nACCION: $(action)\nPRECIO: $(entry_price)\nTP$(index): $(profit_price)\nSL: $(stoploss)",
            "template_type": "market",
            "priority": 100,
        }
        res_create = await client.post("/api/v1/validator/templates", json=new_tpl)
        assert res_create.status_code == 201
        created_tpl = res_create.json()

        # 4. Now parse the signal again (simulating clicking on the signal in /validator)
        res_parse_after = await client.post(
            "/api/v1/validator/parse",
            json={"text": custom_signal_text, "sender_name": "Canal VIP Test"},
        )
        assert res_parse_after.status_code == 200
        after_data = res_parse_after.json()
        assert after_data["templates_tested_count"] == initial_count + 1
        assert after_data["exact_match"] is True
        assert after_data["exact_signals_count"] == 1
        assert after_data["matched_template_id"] == created_tpl["id"]
        assert after_data["matched_template_name"] == "Plantilla Super VIP"
        assert after_data["matched_pattern"] == new_tpl["pattern_syntax"]



