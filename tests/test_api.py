import pytest
from pathlib import Path
from httpx import AsyncClient, ASGITransport
from app.main import app

@pytest.mark.asyncio
async def test_health_endpoint():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        response = await client.get("/health")
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "healthy"
        assert "Rust (PyO3)" in data["engine"]

@pytest.mark.asyncio
async def test_tokenize_endpoint():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        payload = {
            "text": "GBPJPY BUY 199.600\n\nTP 199.700\nTP 199.800\nSL 199.200",
            "persist": True,
        }
        response = await client.post("/api/v1/tokens/tokenize", json=payload)
        assert response.status_code == 200
        data = response.json()
        assert "tokens" in data
        assert len(data["tokens"]) > 0
        assert data["entities"]["has_signal"] is True
        assert data["record_id"] is not None
        assert data["processing_time_us"] > 0

@pytest.mark.asyncio
async def test_templates_crud_and_match():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # Create template
        create_payload = {
            "name": "Integration Test Template",
            "pattern_syntax": "$(symbol) $(action) $(entry_price)\n\nTP $(profit_price)\nSL $(stoploss)",
            "template_type": "market",
            "description": "Integration test template",
            "priority": 200,
        }
        res_create = await client.post("/api/v1/templates", json=create_payload)
        assert res_create.status_code == 201
        tpl_data = res_create.json()
        template_id = tpl_data["id"]

        # List templates
        res_list = await client.get("/api/v1/templates?limit=200")
        assert res_list.status_code == 200
        templates = res_list.json()
        assert any(t["id"] == template_id for t in templates)

        # Match against template
        match_payload = {
            "text": "EURUSD BUY 1.0850\n\nTP 1.0900\nSL 1.0800",
            "template_id": template_id,
        }
        res_match = await client.post("/api/v1/templates/match", json=match_payload)
        assert res_match.status_code == 200
        match_data = res_match.json()
        assert match_data["matched"] is True
        assert match_data["template_id"] == template_id
        assert match_data["signal"]["symbol"] == "EURUSD"
        assert match_data["signal"]["action"] == "BUY"

        # Cleanup created template so test data does not linger in DB
        res_del = await client.delete(f"/api/v1/templates/{template_id}")
        assert res_del.status_code == 200

@pytest.mark.asyncio
async def test_benchmarks_replay_endpoint():
    transport = ASGITransport(app=app)
    candidate_paths = [
        Path("/app/data/trading_signals.db"),
        Path("/Users/macseqoia/gits/app/trading_signals.db"),
        Path("./data/trading_signals.db"),
    ]
    db_path = next((str(p) for p in candidate_paths if p.exists()), None)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        payload = {
            "limit": 50,
            "offset": 0,
            "source_db_path": db_path,
        }
        response = await client.post("/api/v1/benchmarks/replay-signals", json=payload)
        assert response.status_code == 200
        data = response.json()
        assert "total_evaluated" in data
        assert "avg_latency_us" in data
        assert "accuracy_score" in data
        assert data["total_evaluated"] > 0
        assert data["avg_latency_us"] > 0
