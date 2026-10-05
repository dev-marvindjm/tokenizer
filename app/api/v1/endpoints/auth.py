import uuid
from datetime import datetime, timedelta, timezone
from typing import List, Optional, Tuple
from fastapi import APIRouter, Depends, HTTPException, status, Request
from fastapi.security import OAuth2PasswordRequestForm
from sqlmodel import select
from sqlmodel.ext.asyncio.session import AsyncSession

from app.core.config import settings
from app.core.database import get_session
from app.core.security import (
    get_password_hash,
    verify_password,
    create_access_token,
    generate_refresh_token,
    hash_token,
    generate_api_key,
)
from app.core.deps import get_current_user_and_session, get_current_user
from app.domain.models import User, UserSession, ApiKey, utc_now
from app.domain.schemas import (
    UserCreate,
    UserRead,
    UserLoginRequest,
    TokenResponse,
    RefreshTokenRequest,
    UserSessionRead,
    ApiKeyCreate,
    ApiKeyCreatedResponse,
    ApiKeyRead,
)

router = APIRouter(prefix="/auth", tags=["Authentication & Multi-Login"])

@router.post("/register", response_model=UserRead, status_code=status.HTTP_201_CREATED, summary="Register a new user")
async def register_user(
    payload: UserCreate,
    session: AsyncSession = Depends(get_session),
) -> UserRead:
    """Creates a new user account with secure bcrypt password hashing."""
    # Check if username or email already exists
    stmt = select(User).where((User.username == payload.username) | (User.email == payload.email))
    res = await session.exec(stmt)
    if res.first():
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="A user with this username or email already exists.",
        )

    user = User(
        email=payload.email,
        username=payload.username,
        hashed_password=get_password_hash(payload.password),
        full_name=payload.full_name,
        is_active=True,
        is_superuser=False,
    )
    session.add(user)
    await session.commit()
    await session.refresh(user)
    return user


async def _create_user_session_and_tokens(
    user: User,
    device_name: Optional[str],
    request: Request,
    session: AsyncSession,
) -> TokenResponse:
    """Helper to register a new concurrent login session and issue access/refresh tokens."""
    session_id = str(uuid.uuid4())
    raw_refresh_token = generate_refresh_token()
    refresh_hash = hash_token(raw_refresh_token)

    user_agent = request.headers.get("user-agent", "unknown")
    client_ip = request.client.host if request.client else None
    now = utc_now()
    expires_at = now + timedelta(days=settings.REFRESH_TOKEN_EXPIRE_DAYS)

    # Multi-login: create persistent session entry without invalidating existing sessions
    user_session = UserSession(
        user_id=user.id,
        session_id=session_id,
        refresh_token_hash=refresh_hash,
        device_name=device_name or "Unknown Device",
        user_agent=user_agent[:255] if user_agent else None,
        ip_address=client_ip,
        is_active=True,
        expires_at=expires_at,
        last_active_at=now,
        created_at=now,
    )
    session.add(user_session)
    await session.commit()

    # Generate JWT Access Token embedding user ID and session ID
    access_token = create_access_token(
        subject=user.id,
        session_id=session_id,
        extra_claims={"username": user.username, "is_superuser": user.is_superuser},
    )

    return TokenResponse(
        access_token=access_token,
        token_type="bearer",
        expires_in=settings.ACCESS_TOKEN_EXPIRE_MINUTES * 60,
        refresh_token=raw_refresh_token,
        session_id=session_id,
        user=UserRead.model_validate(user),
    )


@router.post("/login", response_model=TokenResponse, summary="Login from any device (supports multi-login)")
async def login(
    payload: UserLoginRequest,
    request: Request,
    session: AsyncSession = Depends(get_session),
) -> TokenResponse:
    """
    Authenticates a user and starts a new concurrent session.
    Multiple logins from different devices (desktop, phone, bot) remain active simultaneously.
    """
    stmt = select(User).where(
        (User.username == payload.username_or_email) | (User.email == payload.username_or_email)
    )
    res = await session.exec(stmt)
    user = res.first()

    if not user or not verify_password(payload.password, user.hashed_password):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Incorrect username/email or password",
        )

    if not user.is_active:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Inactive user account",
        )

    return await _create_user_session_and_tokens(user, payload.device_name, request, session)


@router.post("/token", response_model=TokenResponse, summary="OAuth2 compatible token endpoint")
async def login_for_access_token(
    form_data: OAuth2PasswordRequestForm = Depends(),
    request: Request = None,
    session: AsyncSession = Depends(get_session),
) -> TokenResponse:
    """OAuth2 password flow compatible endpoint (for Swagger UI 'Authorize' button)."""
    stmt = select(User).where(
        (User.username == form_data.username) | (User.email == form_data.username)
    )
    res = await session.exec(stmt)
    user = res.first()

    if not user or not verify_password(form_data.password, user.hashed_password):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Incorrect username or password",
            headers={"WWW-Authenticate": "Bearer"},
        )

    return await _create_user_session_and_tokens(user, "Swagger UI Client", request, session)


@router.post("/refresh", response_model=TokenResponse, summary="Refresh access token using refresh token")
async def refresh_access_token(
    payload: RefreshTokenRequest,
    session: AsyncSession = Depends(get_session),
) -> TokenResponse:
    """Refreshes an expired access token using the active multi-login session's refresh token."""
    token_hash = hash_token(payload.refresh_token)
    now = utc_now()

    stmt = select(UserSession).where(
        UserSession.refresh_token_hash == token_hash,
        UserSession.is_active == True,
    )
    res = await session.exec(stmt)
    user_session = res.first()

    if not user_session or user_session.expires_at <= now:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid or expired refresh token",
        )

    # Fetch user
    user_stmt = select(User).where(User.id == user_session.user_id, User.is_active == True)
    user_res = await session.exec(user_stmt)
    user = user_res.first()

    if not user:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="User not found")

    # Rotate refresh token
    new_refresh_token = generate_refresh_token()
    user_session.refresh_token_hash = hash_token(new_refresh_token)
    user_session.last_active_at = now
    session.add(user_session)
    await session.commit()

    # Generate new access token for the same session
    access_token = create_access_token(
        subject=user.id,
        session_id=user_session.session_id,
        extra_claims={"username": user.username, "is_superuser": user.is_superuser},
    )

    return TokenResponse(
        access_token=access_token,
        token_type="bearer",
        expires_in=settings.ACCESS_TOKEN_EXPIRE_MINUTES * 60,
        refresh_token=new_refresh_token,
        session_id=user_session.session_id,
        user=UserRead.model_validate(user),
    )


@router.get("/me", response_model=UserRead, summary="Get current logged-in user profile")
async def get_my_profile(
    current_user: User = Depends(get_current_user),
) -> UserRead:
    return current_user


@router.get("/sessions", response_model=List[UserSessionRead], summary="List all active sessions for current user (Multi-login monitor)")
async def list_active_sessions(
    auth_tuple: Tuple[User, Optional[str]] = Depends(get_current_user_and_session),
    session: AsyncSession = Depends(get_session),
) -> List[UserSessionRead]:
    """Displays all devices and sessions currently logged in under this user account."""
    current_user, current_sid = auth_tuple
    now = utc_now()

    stmt = (
        select(UserSession)
        .where(
            UserSession.user_id == current_user.id,
            UserSession.is_active == True,
            UserSession.expires_at > now,
        )
        .order_by(UserSession.last_active_at.desc())
    )
    res = await session.exec(stmt)
    sessions = res.all()

    output: List[UserSessionRead] = []
    for s in sessions:
        dto = UserSessionRead.model_validate(s)
        dto.is_current = (s.session_id == current_sid)
        output.append(dto)
    return output


@router.post("/logout", status_code=status.HTTP_200_OK, summary="Logout current session or specified session")
async def logout_session(
    session_id: Optional[str] = None,
    auth_tuple: Tuple[User, Optional[str]] = Depends(get_current_user_and_session),
    session: AsyncSession = Depends(get_session),
) -> dict:
    """Logs out and revokes a specific device session without affecting other active sessions."""
    current_user, current_sid = auth_tuple
    target_sid = session_id or current_sid

    if not target_sid:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="No session identified to logout (API Key access has no session).",
        )

    stmt = select(UserSession).where(
        UserSession.session_id == target_sid,
        UserSession.user_id == current_user.id,
    )
    res = await session.exec(stmt)
    sess_obj = res.first()

    if sess_obj:
        sess_obj.is_active = False
        session.add(sess_obj)
        await session.commit()

    return {"detail": f"Session {target_sid} successfully logged out."}


@router.post("/logout-all", status_code=status.HTTP_200_OK, summary="Revoke all active sessions (Logout from all devices)")
async def logout_all_sessions(
    current_user: User = Depends(get_current_user),
    session: AsyncSession = Depends(get_session),
) -> dict:
    """Revokes every active login session for this user across all computers, phones and bots."""
    stmt = select(UserSession).where(
        UserSession.user_id == current_user.id,
        UserSession.is_active == True,
    )
    res = await session.exec(stmt)
    sessions = res.all()

    revoked_count = 0
    for s in sessions:
        s.is_active = False
        session.add(s)
        revoked_count += 1

    await session.commit()
    return {"detail": f"All {revoked_count} active sessions revoked successfully."}


# ==========================================
# API Keys for Trading Bots
# ==========================================
@router.post("/api-keys", response_model=ApiKeyCreatedResponse, status_code=status.HTTP_201_CREATED, summary="Generate an API key for automated trading bots")
async def create_user_api_key(
    payload: ApiKeyCreate,
    current_user: User = Depends(get_current_user),
    session: AsyncSession = Depends(get_session),
) -> ApiKeyCreatedResponse:
    """Generates an API key (e.g. for MetaTrader 5 bot or Telegram daemon). Key is displayed only once."""
    raw_key, prefix, hashed = generate_api_key(name_hint="bot")
    now = utc_now()
    expires_at = now + timedelta(days=payload.expires_days) if payload.expires_days else None

    api_key_obj = ApiKey(
        user_id=current_user.id,
        name=payload.name,
        prefix=prefix,
        hashed_key=hashed,
        is_active=True,
        expires_at=expires_at,
        created_at=now,
    )
    session.add(api_key_obj)
    await session.commit()
    await session.refresh(api_key_obj)

    return ApiKeyCreatedResponse(
        id=api_key_obj.id,
        name=api_key_obj.name,
        api_key=raw_key,
        prefix=prefix,
        created_at=api_key_obj.created_at,
        expires_at=api_key_obj.expires_at,
    )


@router.get("/api-keys", response_model=List[ApiKeyRead], summary="List all active API keys")
async def list_user_api_keys(
    current_user: User = Depends(get_current_user),
    session: AsyncSession = Depends(get_session),
) -> List[ApiKeyRead]:
    stmt = (
        select(ApiKey)
        .where(ApiKey.user_id == current_user.id)
        .order_by(ApiKey.created_at.desc())
    )
    res = await session.exec(stmt)
    return res.all()


@router.delete("/api-keys/{key_id}", status_code=status.HTTP_200_OK, summary="Revoke an API key")
async def delete_user_api_key(
    key_id: int,
    current_user: User = Depends(get_current_user),
    session: AsyncSession = Depends(get_session),
) -> dict:
    stmt = select(ApiKey).where(ApiKey.id == key_id, ApiKey.user_id == current_user.id)
    res = await session.exec(stmt)
    key_obj = res.first()
    if not key_obj:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="API key not found")

    key_obj.is_active = False
    session.add(key_obj)
    await session.commit()
    return {"detail": f"API key {key_obj.name} ({key_obj.prefix}...) revoked."}
