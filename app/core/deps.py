from datetime import datetime, timezone
from typing import Optional, Tuple
from fastapi import Depends, HTTPException, status, Request
from fastapi.security import OAuth2PasswordBearer, APIKeyHeader
from sqlmodel import select
from sqlmodel.ext.asyncio.session import AsyncSession

from app.core.database import get_session
from app.core.security import decode_access_token, hash_token
from app.domain.models import User, UserSession, ApiKey, utc_now

oauth2_scheme = OAuth2PasswordBearer(tokenUrl="/api/v1/auth/token", auto_error=False)
api_key_header = APIKeyHeader(name="X-API-Key", auto_error=False)

async def get_current_user_and_session(
    request: Request,
    token: Optional[str] = Depends(oauth2_scheme),
    api_key: Optional[str] = Depends(api_key_header),
    session: AsyncSession = Depends(get_session),
) -> Tuple[User, Optional[str]]:
    """
    Authenticates the request using either:
      1) Bearer JWT Access Token with active Session verification (Multi-login)
      2) X-API-Key Header for automated trading bots and external microservices
    Returns:
      (User, session_id or None)
    """
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Could not validate credentials or session expired",
        headers={"WWW-Authenticate": "Bearer"},
    )

    # 1. Try Bearer JWT Token
    if token:
        payload = decode_access_token(token)
        if not payload or "sub" not in payload:
            raise credentials_exception

        try:
            user_id = int(payload["sub"])
        except ValueError:
            raise credentials_exception

        session_id = payload.get("sid")

        # Query user from DB
        user_stmt = select(User).where(User.id == user_id, User.is_active == True)
        res = await session.exec(user_stmt)
        user = res.first()
        if not user:
            raise credentials_exception

        # If a session_id is in token, verify that this specific session is active & unrevoked
        if session_id:
            sess_stmt = select(UserSession).where(
                UserSession.session_id == session_id,
                UserSession.user_id == user.id,
                UserSession.is_active == True,
            )
            sess_res = await session.exec(sess_stmt)
            user_session = sess_res.first()
            if not user_session:
                raise HTTPException(
                    status_code=status.HTTP_401_UNAUTHORIZED,
                    detail="Session has been revoked or logged out",
                    headers={"WWW-Authenticate": "Bearer"},
                )

            # Check session expiry
            now = utc_now()
            if user_session.expires_at <= now:
                user_session.is_active = False
                session.add(user_session)
                await session.commit()
                raise HTTPException(
                    status_code=status.HTTP_401_UNAUTHORIZED,
                    detail="Session has expired. Please login again.",
                    headers={"WWW-Authenticate": "Bearer"},
                )

            # Update last activity timestamp on this session
            user_session.last_active_at = now
            session.add(user_session)
            await session.commit()

        return user, session_id

    # 2. Try X-API-Key
    if api_key:
        hashed = hash_token(api_key)
        now = utc_now()
        key_stmt = select(ApiKey).where(
            ApiKey.hashed_key == hashed,
            ApiKey.is_active == True,
        )
        key_res = await session.exec(key_stmt)
        matched_key = key_res.first()

        if not matched_key:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Invalid or revoked API Key",
            )

        if matched_key.expires_at and matched_key.expires_at <= now:
            matched_key.is_active = False
            session.add(matched_key)
            await session.commit()
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="API Key has expired",
            )

        # Update last used timestamp
        matched_key.last_used_at = now
        session.add(matched_key)
        await session.commit()

        # Query user
        user_stmt = select(User).where(User.id == matched_key.user_id, User.is_active == True)
        user_res = await session.exec(user_stmt)
        user = user_res.first()
        if not user:
            raise credentials_exception

        return user, None

    raise credentials_exception

async def get_current_user(
    auth_tuple: Tuple[User, Optional[str]] = Depends(get_current_user_and_session),
) -> User:
    """Dependency returning the authenticated User model."""
    user, _ = auth_tuple
    return user

async def get_current_superuser(
    user: User = Depends(get_current_user),
) -> User:
    """Dependency verifying that the user has superuser privileges."""
    if not user.is_superuser:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="The user does not have enough privileges",
        )
    return user

async def get_optional_current_user(
    request: Request,
    token: Optional[str] = Depends(oauth2_scheme),
    api_key: Optional[str] = Depends(api_key_header),
    session: AsyncSession = Depends(get_session),
) -> Optional[User]:
    """Dependency returning the authenticated User if valid credentials provided, or None."""
    if not token and not api_key:
        return None
    try:
        user, _ = await get_current_user_and_session(request, token, api_key, session)
        return user
    except HTTPException:
        return None

