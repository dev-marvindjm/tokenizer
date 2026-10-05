from typing import AsyncGenerator, Annotated
from pathlib import Path
from fastapi import Depends
from sqlalchemy.ext.asyncio import create_async_engine, async_sessionmaker
from sqlmodel.ext.asyncio.session import AsyncSession
from sqlmodel import SQLModel

from app.core.config import settings

# Create async engine with pool configuration
engine = create_async_engine(
    settings.DATABASE_URL,
    echo=False,
    future=True,
    connect_args={"check_same_thread": False} if "sqlite" in settings.DATABASE_URL else {},
)

async_session_factory = async_sessionmaker(
    bind=engine,
    class_=AsyncSession,
    expire_on_commit=False,
    autocommit=False,
    autoflush=False,
)

async def init_db() -> None:
    """Create tables if they do not exist."""
    if "sqlite" in settings.DATABASE_URL:
        # Extract path from SQLite connection string
        url = settings.DATABASE_URL
        if ":////" in url:
            db_path = Path("/" + url.split(":////", 1)[1])
        elif ":///" in url:
            db_path = Path(url.split(":///", 1)[1])
        else:
            db_path = Path("./data/trading_signals_app.db")
        db_path.parent.mkdir(parents=True, exist_ok=True)

    async with engine.begin() as conn:
        await conn.run_sync(SQLModel.metadata.create_all)
        # Migrate columns for SQLite if upgrading from previous single-user schema
        if "sqlite" in settings.DATABASE_URL:
            from sqlalchemy import text
            for alter_sql in [
                "ALTER TABLE signal_templates ADD COLUMN user_id INTEGER REFERENCES users(id)",
                "ALTER TABLE signal_templates ADD COLUMN is_system BOOLEAN NOT NULL DEFAULT 0",
                "ALTER TABLE signal_templates ADD COLUMN code_slot VARCHAR",
                "ALTER TABLE signal_templates ADD COLUMN section VARCHAR",
                "ALTER TABLE signal_templates ADD COLUMN sender_name VARCHAR",
                "ALTER TABLE signal_templates ADD COLUMN win_rate FLOAT",
                "ALTER TABLE signal_templates ADD COLUMN latency_us INTEGER",
                "ALTER TABLE signal_templates ADD COLUMN signals_count INTEGER",
                "ALTER TABLE signal_templates ADD COLUMN quality_score INTEGER",
                "ALTER TABLE tokenized_records ADD COLUMN user_id INTEGER REFERENCES users(id)",
            ]:
                try:
                    await conn.execute(text(alter_sql))
                except Exception:
                    pass

async def get_session() -> AsyncGenerator[AsyncSession, None]:
    """Dependency generator for async database sessions."""
    async with async_session_factory() as session:
        try:
            yield session
        finally:
            await session.close()

SessionDep = Annotated[AsyncSession, Depends(get_session)]
