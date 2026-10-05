import hashlib
import secrets
from datetime import datetime, timedelta, timezone
from typing import Optional, Dict, Any, Tuple
import bcrypt
import jwt

from app.core.config import settings

def get_password_hash(password: str) -> str:
    """Hashes a plain text password using bcrypt with automatic salt."""
    salt = bcrypt.gensalt()
    return bcrypt.hashpw(password.encode("utf-8"), salt).decode("utf-8")

def verify_password(plain_password: str, hashed_password: str) -> bool:
    """Verifies a plain password against the stored bcrypt hash."""
    try:
        return bcrypt.checkpw(plain_password.encode("utf-8"), hashed_password.encode("utf-8"))
    except Exception:
        return False

def hash_token(token: str) -> str:
    """Computes SHA-256 hash of a session/refresh token or API key for secure storage."""
    return hashlib.sha256(token.encode("utf-8")).hexdigest()

def create_access_token(
    subject: str | int,
    session_id: str,
    extra_claims: Optional[Dict[str, Any]] = None,
    expires_delta: Optional[timedelta] = None,
) -> str:
    """
    Creates a signed JWT access token.
    Claims:
      - sub: user_id (as str)
      - sid: session_id (unique per active login session)
      - exp: expiration timestamp
      - iat: issued at timestamp
    """
    now = datetime.now(timezone.utc)
    if expires_delta:
        expire = now + expires_delta
    else:
        expire = now + timedelta(minutes=settings.ACCESS_TOKEN_EXPIRE_MINUTES)

    to_encode: Dict[str, Any] = {
        "sub": str(subject),
        "sid": session_id,
        "iat": int(now.timestamp()),
        "exp": int(expire.timestamp()),
    }
    if extra_claims:
        to_encode.update(extra_claims)

    encoded_jwt = jwt.encode(to_encode, settings.SECRET_KEY, algorithm=settings.ALGORITHM)
    return encoded_jwt

def decode_access_token(token: str) -> Optional[Dict[str, Any]]:
    """Decodes and validates a JWT access token."""
    try:
        payload = jwt.decode(token, settings.SECRET_KEY, algorithms=[settings.ALGORITHM])
        return payload
    except (jwt.PyJWTError, ValueError):
        return None

def generate_refresh_token() -> str:
    """Generates a cryptographically secure random refresh token string."""
    return secrets.token_urlsafe(48)

def generate_api_key(name_hint: str = "live") -> Tuple[str, str, str]:
    """
    Generates a high-entropy API key.
    Returns:
      (raw_full_key, prefix, hashed_key)
    Example:
      full_key: tk_live_89f4b238c928...
      prefix: tk_live_89f4b238
      hashed_key: sha256 hex digest
    """
    random_part = secrets.token_hex(24)
    raw_key = f"{settings.API_KEY_PREFIX}{name_hint}_{random_part}"
    prefix = raw_key[:16]
    hashed = hash_token(raw_key)
    return raw_key, prefix, hashed
