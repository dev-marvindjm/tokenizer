from datetime import datetime, timezone
from typing import Optional, Dict, Any, List
from sqlmodel import SQLModel, Field, Relationship
from sqlalchemy import Column, JSON

def utc_now() -> datetime:
    return datetime.now(timezone.utc)

class User(SQLModel, table=True):
    __tablename__ = "users"

    id: Optional[int] = Field(default=None, primary_key=True)
    email: str = Field(unique=True, index=True, nullable=False)
    username: str = Field(unique=True, index=True, nullable=False)
    hashed_password: str = Field(nullable=False)
    full_name: Optional[str] = Field(default=None)
    is_active: bool = Field(default=True, index=True)
    is_superuser: bool = Field(default=False)
    created_at: datetime = Field(default_factory=utc_now, nullable=False)
    updated_at: datetime = Field(default_factory=utc_now, nullable=False)

    sessions: List["UserSession"] = Relationship(back_populates="user", cascade_delete=True)
    api_keys: List["ApiKey"] = Relationship(back_populates="user", cascade_delete=True)
    templates: List["SignalTemplate"] = Relationship(back_populates="user")
    records: List["TokenizedRecord"] = Relationship(back_populates="user")


class UserSession(SQLModel, table=True):
    __tablename__ = "user_sessions"

    id: Optional[int] = Field(default=None, primary_key=True)
    user_id: int = Field(foreign_key="users.id", index=True, nullable=False)
    session_id: str = Field(unique=True, index=True, nullable=False)
    refresh_token_hash: str = Field(nullable=False)
    device_name: Optional[str] = Field(default=None)
    user_agent: Optional[str] = Field(default=None)
    ip_address: Optional[str] = Field(default=None)
    is_active: bool = Field(default=True, index=True)
    expires_at: datetime = Field(nullable=False, index=True)
    last_active_at: datetime = Field(default_factory=utc_now, nullable=False)
    created_at: datetime = Field(default_factory=utc_now, nullable=False)

    user: Optional[User] = Relationship(back_populates="sessions")


class ApiKey(SQLModel, table=True):
    __tablename__ = "api_keys"

    id: Optional[int] = Field(default=None, primary_key=True)
    user_id: int = Field(foreign_key="users.id", index=True, nullable=False)
    name: str = Field(nullable=False)
    prefix: str = Field(nullable=False)
    hashed_key: str = Field(unique=True, index=True, nullable=False)
    is_active: bool = Field(default=True, index=True)
    expires_at: Optional[datetime] = Field(default=None)
    last_used_at: Optional[datetime] = Field(default=None)
    created_at: datetime = Field(default_factory=utc_now, nullable=False)

    user: Optional[User] = Relationship(back_populates="api_keys")


class SignalTemplate(SQLModel, table=True):
    __tablename__ = "signal_templates"

    id: Optional[int] = Field(default=None, primary_key=True)
    user_id: Optional[int] = Field(default=None, foreign_key="users.id", index=True, nullable=True)
    is_system: bool = Field(default=False, index=True)
    name: str = Field(index=True, nullable=False)
    pattern_syntax: str = Field(nullable=False)
    template_type: str = Field(default="market", nullable=False)
    description: Optional[str] = Field(default=None)
    example_text: Optional[str] = Field(default=None)
    priority: int = Field(default=100)
    is_active: bool = Field(default=True, index=True)
    code_slot: Optional[str] = Field(default=None, index=True)
    section: Optional[str] = Field(default=None, index=True)
    sender_name: Optional[str] = Field(default=None)
    win_rate: Optional[float] = Field(default=None)
    latency_us: Optional[int] = Field(default=None)
    signals_count: Optional[int] = Field(default=None)
    quality_score: Optional[int] = Field(default=None)
    created_at: datetime = Field(default_factory=utc_now, nullable=False)
    updated_at: datetime = Field(default_factory=utc_now, nullable=False)

    user: Optional[User] = Relationship(back_populates="templates")
    records: List["TokenizedRecord"] = Relationship(back_populates="template")


class TokenizedRecord(SQLModel, table=True):
    __tablename__ = "tokenized_records"

    id: Optional[int] = Field(default=None, primary_key=True)
    user_id: Optional[int] = Field(default=None, foreign_key="users.id", index=True, nullable=True)
    template_id: Optional[int] = Field(default=None, foreign_key="signal_templates.id", nullable=True, index=True)
    raw_text: str = Field(nullable=False)
    tokens_json: Dict[str, Any] = Field(default_factory=dict, sa_column=Column(JSON))
    extracted_entities: Dict[str, Any] = Field(default_factory=dict, sa_column=Column(JSON))
    processing_time_us: int = Field(default=0, nullable=False)
    created_at: datetime = Field(default_factory=utc_now, nullable=False, index=True)

    user: Optional[User] = Relationship(back_populates="records")
    template: Optional[SignalTemplate] = Relationship(back_populates="records")


class TemplateConfig(SQLModel, table=True):
    __tablename__ = "template_config"

    id: Optional[int] = Field(default=None, primary_key=True)
    template_id: int = Field(unique=True, index=True, nullable=False)
    strategy_id: Optional[int] = Field(default=None)
    ammount_type: str = Field(default="percentage", nullable=False)
    ammount: float = Field(default=1.0, nullable=False)
    exit_strategy_type: str = Field(default="gale", nullable=False)
    exit_strategy_id: Optional[int] = Field(default=None)
    exit_contained_inside: bool = Field(default=False, nullable=False)
    max_gale: Optional[int] = Field(default=0)
    gale_ammount: Optional[float] = Field(default=2.0)
    profit_ratio: Optional[float] = Field(default=None)
    loss_type: Optional[str] = Field(default="price")
    loss_ratio: Optional[float] = Field(default=None)
    logic: Optional[str] = Field(default=None)
    created_at: datetime = Field(default_factory=utc_now, nullable=False)
    updated_at: datetime = Field(default_factory=utc_now, nullable=False)


class TemplateBroker(SQLModel, table=True):
    __tablename__ = "template_brokers"

    id: Optional[int] = Field(default=None, primary_key=True)
    template_id: int = Field(index=True, nullable=False)
    broker_id: int = Field(nullable=False)
    broker_account_id: Optional[int] = Field(default=None)
    broker_name: Optional[str] = Field(default=None, index=True)
    account_number: Optional[str] = Field(default=None, index=True)
    is_active: bool = Field(default=True, nullable=False)
    timezone: Optional[str] = Field(default="UTC-3")
    created_at: datetime = Field(default_factory=utc_now, nullable=False)
    updated_at: datetime = Field(default_factory=utc_now, nullable=False)
