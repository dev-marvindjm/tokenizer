# Graph Report - tokenizer  (2026-10-19)

## Corpus Check
- 49 files · ~447,556 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 7 file(s) not represented in the graph (top: .db 4, (none) 3)

## Summary
- 581 nodes · 1263 edges · 40 communities (22 shown, 18 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 93 edges (avg confidence: 0.93)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Rust Token Matchers & Lexicon
- Validator & Signal Replay API
- Authentication & Session Endpoints
- FastAPI Application & Integration Tests
- Tokenizer & Benchmarks API Routes
- Signal Templates & Brokers API
- Pattern Matcher Flow Control
- Token Classification Enums
- PyO3 Rust Bindings & Entities
- Rust Token Stream Iterators
- Database & Auth Dependencies
- JWT & Password Security Utilities
- Python-to-Rust Bridge Service
- Financial Symbol Parsing & Types
- Template Scoring & Timezone Logic
- Signal Serialization & JSON Templates
- Signal Timeframe & Price Parsing
- Template Matching & Unit Tests
- Price & Time Entry Validation
- Profit/Loss Targets & Signal Types
- Validator Frontend UI Workflow
- Custom Tokens & Categories API
- OTC & Perpetual Symbol Classes
- Docker Compose & Python Dependencies
- Index Symbol Validation Datasets
- Timezone Validation Datasets
- Container Entrypoint Script
- Validator UI Export Helper
- Ground Truth Studio UI
- Validator UI Senders Loader
- Validator UI Templates Loader
- Rust Core Native Module
- Crypto Symbols Dataset
- ETF Symbols Dataset
- Forex Symbols Dataset
- Fund Symbols Dataset
- Money Market Symbols Dataset
- Stock Symbols Dataset
- Minute Timeframe Dataset
- Rust Core Benchmark Binding

## God Nodes (most connected - your core abstractions)
1. `Nodes` - 45 edges
2. `Patterns` - 33 edges
3. `User` - 31 edges
4. `TokenClass` - 28 edges
5. `parse_signal()` - 28 edges
6. `Signal` - 23 edges
7. `parse_template_scored()` - 20 edges
8. `SignalTemplate` - 15 edges
9. `parser()` - 15 edges
10. `_create_user_session_and_tokens()` - 14 edges

## Surprising Connections (you probably didn't know these)
- `runParse` --conceptually_related_to--> `Signal Keywords & Emoji Dictionary (info_v)`  [INFERRED]
  app/static/validator.html → data/dataset/info_v.txt
- `create_user_api_key()` --uses--> `ApiKey`  [INFERRED]
  app/api/v1/endpoints/auth.py → app/domain/models.py
- `list_user_api_keys()` --uses--> `ApiKey`  [INFERRED]
  app/api/v1/endpoints/auth.py → app/domain/models.py
- `delete_user_api_key()` --uses--> `ApiKey`  [INFERRED]
  app/api/v1/endpoints/auth.py → app/domain/models.py
- `replay_signals_endpoint()` --uses--> `ReplayBenchmarkRequest`  [INFERRED]
  app/api/v1/endpoints/benchmarks.py → app/domain/schemas.py

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Financial Asset Symbol Validation Datasets** — data_dataset_symbol_crypto_val_dataset, data_dataset_symbol_etf_val_dataset, data_dataset_symbol_forex_val_dataset, data_dataset_symbol_fund_val_dataset, data_dataset_symbol_index_val_dataset, data_dataset_symbol_index_val_parser_dataset, data_dataset_symbol_money_val_dataset, data_dataset_symbol_stock_val_dataset [INFERRED 0.95]
- **Signal Temporal & Keyword Lexicon Datasets** — data_dataset_info_v_dataset, data_dataset_timeframe_val_m_dataset, data_dataset_timezone_val_d_dataset, data_dataset_timezone_val_v_dataset [INFERRED 0.95]
- **Ground Truth Signal Validation UI Workflow** — app_static_validator_ground_truth_studio, app_static_validator_loadmessages, app_static_validator_selectmessage, app_static_validator_runparse, app_static_validator_savevalidation, app_static_validator_batchmarknosignal, app_static_validator_getexporturl [EXTRACTED 1.00]

## Communities (40 total, 18 thin omitted)

### Community 0 - "Rust Token Matchers & Lexicon"
Cohesion: 0.05
Nodes (74): ACTION_V, ACTION_VAL_D, ACTION_VAL_U, CHECKER_NODES, DATE_VAL, DIGIT, EMOJI, EMOJI_CHARS (+66 more)

### Community 1 - "Validator & Signal Replay API"
Cohesion: 0.09
Nodes (22): replay_signals_endpoint(), batch_validate_messages(), BatchValidateRequest, export_tests(), get_db_path(), get_message_detail(), get_messages(), get_senders() (+14 more)

### Community 2 - "Authentication & Session Endpoints"
Cohesion: 0.13
Nodes (25): create_user_api_key(), _create_user_session_and_tokens(), delete_user_api_key(), get_my_profile(), list_active_sessions(), list_user_api_keys(), login(), login_for_access_token() (+17 more)

### Community 3 - "FastAPI Application & Integration Tests"
Cohesion: 0.07
Nodes (21): health_check(), serve_validator_ui(), test_benchmarks_replay_endpoint(), test_health_endpoint(), test_templates_crud_and_match(), test_tokenize_endpoint(), test_auth_registration_and_duplicate(), test_bot_api_key_authentication() (+13 more)

### Community 4 - "Tokenizer & Benchmarks API Routes"
Cohesion: 0.11
Nodes (10): tokenize_endpoint(), api_v1_health(), Settings, init_db(), get_password_hash(), TokenDTO, TokenizeRequest, TokenizeResponse (+2 more)

### Community 5 - "Signal Templates & Brokers API"
Cohesion: 0.16
Nodes (22): batch_toggle_templates(), create_template(), delete_template(), get_brokers_catalog(), get_template(), get_template_brokers(), get_template_config(), link_template_broker() (+14 more)

### Community 6 - "Pattern Matcher Flow Control"
Cohesion: 0.11
Nodes (22): check_category(), FlowControlItem, Increment, IncrementAtLeast, IncrementExact, IncrementManyTimes, IncrementTimes, MatcherItem (+14 more)

### Community 7 - "Token Classification Enums"
Cohesion: 0.07
Nodes (27): TokenClass, Action, Emoji, Gale, Indicator, Info, NewLine, Otc (+19 more)

### Community 8 - "PyO3 Rust Bindings & Entities"
Cohesion: 0.10
Nodes (6): extract_entities(), rust_core(), classify_word(), Token, tokenize_text(), tokenize_text_internal()

### Community 9 - "Rust Token Stream Iterators"
Cohesion: 0.15
Nodes (9): str, String, test_zip_token_iterator(), TokenIterator, TokenIterator<'a>, Tokens, zip_tokens(), ZipTokenIterator (+1 more)

### Community 10 - "Database & Auth Dependencies"
Cohesion: 0.14
Nodes (8): get_session(), get_current_superuser(), get_current_user(), get_current_user_and_session(), get_optional_current_user(), ApiKey, TokenizedRecord, utc_now()

### Community 11 - "JWT & Password Security Utilities"
Cohesion: 0.15
Nodes (6): create_access_token(), decode_access_token(), generate_api_key(), generate_refresh_token(), hash_token(), verify_password()

### Community 13 - "Financial Symbol Parsing & Types"
Cohesion: 0.18
Nodes (14): EntryRange, get_symbol_from_text(), Symbol, SymbolType, Crypto, Empty, Etf, Forex (+6 more)

### Community 14 - "Template Scoring & Timezone Logic"
Cohesion: 0.19
Nodes (8): Gale, NoGale, Number, TimeEntry, is_line_match(), parse_template_scored(), Signal, TemplateRequirements

### Community 15 - "Signal Serialization & JSON Templates"
Cohesion: 0.21
Nodes (4): match_template(), parse_signal_json(), parse_template_json(), ParsedSignal

### Community 16 - "Signal Timeframe & Price Parsing"
Cohesion: 0.22
Nodes (8): TimePrice, parse_signal(), PeriodTime, test_first_symbol_start_line_relative(), test_parse_signal_with_custom_checkers(), test_signal_encapsulation(), test_signal_timezone_fallback(), test_signals_is_binary_and_template_type()

### Community 17 - "Template Matching & Unit Tests"
Cohesion: 0.33
Nodes (10): find_symbol_position(), parse_template(), test_parse_template(), test_parse_template_against_database_examples(), test_parse_template_with_positional_metadata(), test_single_tp_and_dismiss_templates(), test_template_with_otc_and_parentheses_otc(), test_trade_17_single_signal_with_duplicate_action() (+2 more)

### Community 18 - "Price & Time Entry Validation"
Cohesion: 0.20
Nodes (3): Entry, PriceEntry, TimeEntry

### Community 19 - "Profit/Loss Targets & Signal Types"
Cohesion: 0.25
Nodes (4): PricesTarget, Signals, Target, ProfitLoss

### Community 20 - "Validator Frontend UI Workflow"
Cohesion: 0.32
Nodes (8): batchMarkNoSignal, loadMessages, loadStats, navMessage, runParse, saveValidation, selectMessage, Signal Keywords & Emoji Dictionary (info_v)

### Community 22 - "OTC & Perpetual Symbol Classes"
Cohesion: 0.50
Nodes (3): SymbolClass, Otc, Perpetual

### Community 23 - "Docker Compose & Python Dependencies"
Cohesion: 0.67
Nodes (3): tokenizer_data Docker Volume, trading-tokenizer-api Docker Service, Python Runtime & Build Dependencies (FastAPI, SQLModel, Maturin)

## Knowledge Gaps
- **55 isolated node(s):** `rust_core`, `rust_core`, `Action`, `Gale`, `Indicator` (+50 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 163 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **18 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `TokenClass` connect `Token Classification Enums` to `Rust Token Matchers & Lexicon`?**
  _High betweenness centrality (0.042) - this node is a cross-community bridge._
- **Are the 23 inferred relationships involving `User` (e.g. with `create_user_api_key()` and `_create_user_session_and_tokens()`) actually correct?**
  _`User` has 23 INFERRED edges - model-reasoned connections that need verification._
- **What connects `rust_core`, `rust_core`, `Action` to the rest of the system?**
  _55 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Rust Token Matchers & Lexicon` be split into smaller, more focused modules?**
  _Cohesion score 0.050073637702503684 - nodes in this community are weakly interconnected._
- **Why does `Nodes` connect `Rust Token Matchers & Lexicon` to `Signal Timeframe & Price Parsing`, `Financial Symbol Parsing & Types`?**
  _High betweenness centrality (0.032) - this node is a cross-community bridge._
- **Should `Validator & Signal Replay API` be split into smaller, more focused modules?**
  _Cohesion score 0.08788159111933395 - nodes in this community are weakly interconnected._
- **Why does `parse_signal()` connect `Signal Timeframe & Price Parsing` to `Rust Token Matchers & Lexicon`, `PyO3 Rust Bindings & Entities`, `Financial Symbol Parsing & Types`, `Template Scoring & Timezone Logic`, `Signal Serialization & JSON Templates`, `Template Matching & Unit Tests`, `Price & Time Entry Validation`, `Profit/Loss Targets & Signal Types`?**
  _High betweenness centrality (0.030) - this node is a cross-community bridge._