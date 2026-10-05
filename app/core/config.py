from pydantic_settings import BaseSettings, SettingsConfigDict
from typing import Optional
from pathlib import Path

class Settings(BaseSettings):
    PROJECT_NAME: str = "Trading Signals Tokenizer & Matcher API"
    VERSION: str = "1.0.0"
    API_V1_STR: str = "/api/v1"

    # Async Database URL: defaults to SQLite async, can be overridden with postgresql+asyncpg://...
    DATABASE_URL: str = "sqlite+aiosqlite:///./data/trading_signals_app.db"

    # Reference paths for data and historical database
    DATA_DIR: Path = Path("./data")
    APP_HISTORICAL_DB_PATH: Optional[str] = "/Users/macseqoia/gits/app/trading_signals.db"

    # Authentication & Security
    SECRET_KEY: str = "trading-signals-tokenizer-super-secret-jwt-key-2026-production"
    ALGORITHM: str = "HS256"
    ACCESS_TOKEN_EXPIRE_MINUTES: int = 120  # 2 hours per login session
    REFRESH_TOKEN_EXPIRE_DAYS: int = 30     # 30 days persistent multi-login
    API_KEY_PREFIX: str = "tk_"

    # Initial System / Admin Credentials
    FIRST_SUPERUSER_EMAIL: str = "admin@quant.local"
    FIRST_SUPERUSER_USERNAME: str = "admin"
    FIRST_SUPERUSER_PASSWORD: str = "admin123456"

    model_config = SettingsConfigDict(env_file=".env", env_file_encoding="utf-8", extra="ignore")

settings = Settings()
