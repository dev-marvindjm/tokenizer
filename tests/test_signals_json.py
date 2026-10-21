import json
import pytest
from pathlib import Path
import rust_core

SIGNALS_JSON_PATHS = [
    Path("./data/verified_signals_test.json"),
    Path("/Users/macseqoia/gits/others/app/test/signals.json"),
    Path("/Users/macseqoia/gits/app/test/signals.json"),
    Path("/app/data/signals.json"),
    Path("./data/signals.json"),
    Path("./test/signals.json"),
]

def load_signals_test_cases():
    for p in SIGNALS_JSON_PATHS:
        if p.exists():
            with open(p, "r", encoding="utf-8") as f:
                data = json.load(f)
            cases = []
            if "dataCases" in data:
                for provider, case_list in data.get("dataCases", {}).items():
                    for idx, c in enumerate(case_list):
                        cases.append((f"{provider}_{idx}", c))
                if cases:
                    return cases
            elif "cases" in data:
                for c in data.get("cases", []):
                    gt = c.get("ground_truth", [])
                    sigs = gt if isinstance(gt, list) else [gt]
                    cases.append((f"{c.get('sender_name', 'case')}_{c.get('id', 0)}", {
                        "text": c.get("text", ""),
                        "signals": sigs
                    }))
                if cases:
                    return cases
    return []

test_cases = load_signals_test_cases()

@pytest.mark.skipif(not test_cases, reason="signals.json not found")
@pytest.mark.parametrize("case_id,case", test_cases)
def test_signal_case(case_id, case):
    raw_text = case["text"]
    expected_signals = case.get("signals", [])

    # Tokenize with Rust core
    tokens = rust_core.tokenize_text(raw_text)
    assert isinstance(tokens, list)
    assert len(tokens) > 0, f"Tokenization returned empty for {case_id}"

    # Extract entities with Rust core
    entities = rust_core.extract_entities(raw_text)
    assert isinstance(entities, dict)

    if expected_signals:
        assert entities["has_signal"] is True, f"Signal expected but has_signal is False for {case_id}"
        extracted = entities.get("signals", [])
        assert len(extracted) >= 1, f"Expected at least 1 signal extracted for {case_id}"

        first_expected = expected_signals[0]
        first_extracted = extracted[0]

        # Verify Action
        if "action" in first_expected and first_expected["action"]:
            if isinstance(first_expected["action"], str):
                expected_dir = first_expected["action"]
            else:
                expected_dir = "BUY" if first_expected["action"].get("Direction") else "SELL"
            assert first_extracted.get("action") == expected_dir, (
                f"Action mismatch for {case_id}: expected {expected_dir}, got {first_extracted.get('action')}"
            )

        # Verify Symbol
        if "symbol" in first_expected and first_expected["symbol"]:
            if isinstance(first_expected["symbol"], str):
                expected_asset = first_expected["symbol"].replace("-OTC", "").upper()
            else:
                expected_asset = first_expected["symbol"].get("Asset", "").replace("-OTC", "").upper()
            if first_extracted.get("symbol"):
                extracted_asset = first_extracted["symbol"].get("asset", "").upper()
                assert expected_asset in extracted_asset or extracted_asset in expected_asset, (
                    f"Symbol mismatch for {case_id}: expected {expected_asset}, got {extracted_asset}"
                )

        # Verify Stop Loss if present
        prices = first_expected.get("prices")
        if prices and prices.get("Stoploss"):
            expected_sl = float(prices["Stoploss"])
            extracted_sl = first_extracted.get("stoploss")
            if extracted_sl is not None:
                assert abs(extracted_sl - expected_sl) < 0.05, (
                    f"SL mismatch for {case_id}: expected {expected_sl}, got {extracted_sl}"
                )

        # Verify Take Profit if present
        if prices and prices.get("Profits"):
            expected_profits = [float(p) for p in prices["Profits"]]
            extracted_profits = first_extracted.get("profits", [])
            if extracted_profits:
                # Check first profit target
                assert abs(extracted_profits[0] - expected_profits[0]) < 0.05, (
                    f"TP1 mismatch for {case_id}: expected {expected_profits[0]}, got {extracted_profits[0]}"
                )
