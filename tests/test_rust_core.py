import rust_core

def test_rust_core_basic_tokenization():
    text = "BTCUSDT BUY 64500 TP 65000 SL 64000"
    tokens = rust_core.tokenize_text(text)
    assert len(tokens) > 0

    kinds = [t.kind for t in tokens]
    assert "SYMBOL" in kinds
    assert "ACTION_BUY" in kinds

def test_rust_core_unicode_and_emojis():
    # Text loaded with emojis, cyrillic, ads, and weird formatting
    text = "🔥🚀 ¡SEÑAL VIP! 🪙 ETHUSDT 🟢 COMPRA @ 3450.50 🎯 TP: 3500 🛑 SL: 3400.00 📊 Éxito garantizado! 💎"
    tokens = rust_core.tokenize_text(text)
    assert len(tokens) > 0

    entities = rust_core.extract_entities(text)
    assert entities["has_signal"] is True
    sigs = entities["signals"]
    assert len(sigs) >= 1
    assert sigs[0]["symbol"]["asset"] == "ETHUSDT"
    assert sigs[0]["action"] == "BUY"

def test_rust_core_empty_and_noise_text():
    # Pure noise or advert without signal
    text = "Únete a nuestro canal VIP para ganar dinero todos los días. Contacto @admin."
    tokens = rust_core.tokenize_text(text)
    assert isinstance(tokens, list)

    entities = rust_core.extract_entities(text)
    assert entities["has_signal"] is False

def test_rust_core_match_template():
    pattern = "$(symbol) $(action) $(entry_price)\n\nTP $(profit_price)\nSL $(stoploss)"
    text = "EURUSD BUY 1.0850\n\nTP 1.0900\nSL 1.0800"

    matched = rust_core.match_template(text, pattern, "Forex_TP_SL")
    assert matched is not None
    assert matched.symbol == "EURUSD"
    assert matched.action == "BUY"
    assert matched.template_name == "Forex_TP_SL"

def test_rust_core_optional_underscore_token():
    pattern = "$(symbol) $(action) $(entry_price) $(_)\nTP $(profit_price)\nSL $(stoploss)"
    text_present = "EURUSD BUY 1.0850 NOW\nTP 1.0900\nSL 1.0800"
    text_absent = "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800"

    assert rust_core.parse_template_exact_json(text_present, pattern, None) is not None
    assert rust_core.parse_template_exact_json(text_absent, pattern, None) is not None

    # $(_line) matches an optional entire single line (can have multiple words)
    pattern_line = "$(symbol) $(action) $(entry_price)\n$(_line)\nTP $(profit_price)\nSL $(stoploss)"
    text_line_present = "EURUSD BUY 1.0850\nCANAL VIP EXTRA\nTP 1.0900\nSL 1.0800"
    text_line_absent = "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800"

    assert rust_core.parse_template_exact_json(text_line_present, pattern_line, None) is not None
    assert rust_core.parse_template_exact_json(text_line_absent, pattern_line, None) is not None

    # $(_lines) matches all remaining lines (0, 1, or multiple lines)
    pattern_lines = "$(symbol) $(action) $(entry_price)\nTP $(profit_price)\nSL $(stoploss)\n$(_lines)"
    text_lines_multi = "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800\nFOOTER LINE 1\nFOOTER LINE 2\nFOOTER LINE 3"

    assert rust_core.parse_template_exact_json(text_line_absent, pattern_lines, None) is not None
    assert rust_core.parse_template_exact_json(text_lines_multi, pattern_lines, None) is not None


