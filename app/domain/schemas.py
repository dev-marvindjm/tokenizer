from typing import Optional, List, Dict, Any
from datetime import datetime
from pydantic import BaseModel, Field, ConfigDict, EmailStr

# ==========================================
# Auth & User Schemas
# ==========================================
class UserCreate(BaseModel):
    email: str = Field(..., max_length=255)
    username: str = Field(..., min_length=3, max_length=50)
    password: str = Field(..., min_length=6, max_length=128)
    full_name: Optional[str] = None

class UserRead(BaseModel):
    id: int
    email: str
    username: str
    full_name: Optional[str] = None
    is_active: bool
    is_superuser: bool
    created_at: datetime

    model_config = ConfigDict(from_attributes=True)

class UserLoginRequest(BaseModel):
    username_or_email: str
    password: str
    device_name: Optional[str] = Field(default=None, description="e.g. Chrome Mac, Mobile Android, VPS Ingestor")

class TokenResponse(BaseModel):
    access_token: str
    token_type: str = "bearer"
    expires_in: int
    refresh_token: str
    session_id: str
    user: UserRead

class RefreshTokenRequest(BaseModel):
    refresh_token: str

class UserSessionRead(BaseModel):
    id: int
    session_id: str
    device_name: Optional[str] = None
    user_agent: Optional[str] = None
    ip_address: Optional[str] = None
    is_active: bool
    expires_at: datetime
    last_active_at: datetime
    created_at: datetime
    is_current: bool = False

    model_config = ConfigDict(from_attributes=True)

class ApiKeyCreate(BaseModel):
    name: str = Field(..., min_length=2, max_length=100, description="Friendly label e.g. 'MT5 Bot'")
    expires_days: Optional[int] = Field(default=None, ge=1, le=365)

class ApiKeyCreatedResponse(BaseModel):
    id: int
    name: str
    api_key: str = Field(..., description="Full secret API key. Store safely, shown only once!")
    prefix: str
    created_at: datetime
    expires_at: Optional[datetime] = None

class ApiKeyRead(BaseModel):
    id: int
    name: str
    prefix: str
    is_active: bool
    last_used_at: Optional[datetime] = None
    created_at: datetime
    expires_at: Optional[datetime] = None

    model_config = ConfigDict(from_attributes=True)


# ==========================================
# Token Schemas
# ==========================================
class TokenDTO(BaseModel):
    kind: str
    value: str
    start: int
    end: int
    line: int

class TokenizeRequest(BaseModel):
    text: str = Field(..., description="Raw signal text to tokenize")
    persist: bool = Field(default=False, description="Whether to persist the result in tokenized_records")
    template_id: Optional[int] = Field(default=None, description="Optional associated template ID")

class TokenizeResponse(BaseModel):
    tokens: List[TokenDTO]
    entities: Dict[str, Any]
    processing_time_us: int
    record_id: Optional[int] = None


# ==========================================
# Template Schemas
# ==========================================
class SignalTemplateCreate(BaseModel):
    name: str = Field(..., max_length=255)
    pattern_syntax: str = Field(...)
    template_type: str = Field(default="market")
    description: Optional[str] = None
    example_text: Optional[str] = None
    priority: int = Field(default=100)
    is_active: bool = Field(default=True)
    is_system: bool = Field(default=False, description="System templates are globally visible (superusers only)")
    code_slot: Optional[str] = None
    section: Optional[str] = None
    sender_name: Optional[str] = None
    win_rate: Optional[float] = None
    latency_us: Optional[int] = None
    signals_count: Optional[int] = None
    quality_score: Optional[int] = None

class SignalTemplateUpdate(BaseModel):
    name: Optional[str] = None
    pattern_syntax: Optional[str] = None
    template_type: Optional[str] = None
    description: Optional[str] = None
    example_text: Optional[str] = None
    priority: Optional[int] = None
    is_active: Optional[bool] = None
    code_slot: Optional[str] = None
    section: Optional[str] = None
    sender_name: Optional[str] = None
    win_rate: Optional[float] = None
    latency_us: Optional[int] = None
    signals_count: Optional[int] = None
    quality_score: Optional[int] = None

class SignalTemplateRead(BaseModel):
    id: int
    user_id: Optional[int] = None
    is_system: bool = False
    name: str
    pattern_syntax: str
    template_type: str
    description: Optional[str] = None
    example_text: Optional[str] = None
    priority: int
    is_active: bool
    code_slot: Optional[str] = None
    section: Optional[str] = None
    sender_name: Optional[str] = None
    win_rate: Optional[float] = None
    latency_us: Optional[int] = None
    signals_count: Optional[int] = None
    quality_score: Optional[int] = None
    created_at: datetime
    updated_at: datetime

    model_config = ConfigDict(from_attributes=True)


# ==========================================
# Match Schemas
# ==========================================
class MatchTemplateRequest(BaseModel):
    text: str = Field(..., description="Raw text to match against templates")
    template_id: Optional[int] = Field(default=None, description="Specific template ID to test against, or None to match against all active accessible templates")

class MatchTemplateResponse(BaseModel):
    matched: bool
    template_id: Optional[int] = None
    template_name: Optional[str] = None
    matched_pattern: Optional[str] = None
    signal: Optional[Dict[str, Any]] = None
    processing_time_us: int


# ==========================================
# Benchmark & Replay Schemas
# ==========================================
class ReplayBenchmarkRequest(BaseModel):
    limit: int = Field(default=200, ge=1, le=6000, description="Max messages to replay")
    offset: int = Field(default=0, ge=0)
    source_db_path: Optional[str] = Field(default=None, description="Path to SQLite database containing source_messages")

class ReplayBenchmarkResponse(BaseModel):
    total_evaluated: int
    matched_count: int
    accuracy_score: float = Field(..., description="Percentage of successfully parsed or matched signals")
    avg_latency_us: float = Field(..., description="Average latency in microseconds")
    min_latency_us: float
    max_latency_us: float
    p95_latency_us: float
    rust_vs_python_speedup_factor: Optional[float] = None
    sample_signals: List[Dict[str, Any]] = Field(default_factory=list)
