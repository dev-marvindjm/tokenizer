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
        res = await client.get("/api/v1/validator/messages?page=1&page_size=10")
        assert res.status_code == 200
        data = res.json()
        assert len(data["items"]) == 10
        assert data["page"] == 1
        assert data["total"] > 4000
        first = data["items"][0]
        assert "raw_text" in first
        assert "sender_name" in first

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
