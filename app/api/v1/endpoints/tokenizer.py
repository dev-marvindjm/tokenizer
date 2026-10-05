from typing import Optional
from fastapi import APIRouter, HTTPException, status, Depends
from app.core.database import SessionDep
from app.core.deps import get_optional_current_user
from app.domain.models import TokenizedRecord, SignalTemplate, User
from app.domain.schemas import TokenizeRequest, TokenizeResponse, TokenDTO
from app.services.rust_bridge import rust_bridge

router = APIRouter(prefix="/tokens", tags=["Tokens & Lexical Analysis"])

@router.post("/tokenize", response_model=TokenizeResponse, summary="Tokenize raw signal text via Rust native core")
async def tokenize_endpoint(
    payload: TokenizeRequest,
    session: SessionDep,
    current_user: Optional[User] = Depends(get_optional_current_user),
) -> TokenizeResponse:
    """
    Tokenizes raw trading signal text using the high-performance native Rust engine (`rust_core`),
    extracts structured trading entities, and optionally persists the record in the database tagged with user_id.
    """
    if not payload.text.strip():
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="The 'text' field cannot be empty.",
        )

    # Validate template existence if template_id is provided
    if payload.template_id is not None:
        template = await session.get(SignalTemplate, payload.template_id)
        if not template:
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail=f"SignalTemplate with id {payload.template_id} not found.",
            )

    # Perform native Rust tokenization and entity extraction concurrently
    raw_tokens, dur_tok_us = await rust_bridge.tokenize_async(payload.text)
    entities, dur_ent_us = await rust_bridge.extract_entities_async(payload.text)
    total_us = dur_tok_us + dur_ent_us

    record_id = None
    if payload.persist:
        record = TokenizedRecord(
            user_id=current_user.id if current_user else None,
            template_id=payload.template_id,
            raw_text=payload.text,
            tokens_json={"tokens": raw_tokens},
            extracted_entities=entities,
            processing_time_us=total_us,
        )
        session.add(record)
        await session.commit()
        await session.refresh(record)
        record_id = record.id

    return TokenizeResponse(
        tokens=[TokenDTO(**t) for t in raw_tokens],
        entities=entities,
        processing_time_us=total_us,
        record_id=record_id,
    )


@router.get("/categories", summary="Get token categories and counts")
async def get_token_categories():
    """Returns available trading token categories and their token counts."""
    return [
        {"id": "actions", "name": "Actions", "count": 14, "description": "Trading action triggers (BUY, SELL, CALL, PUT, etc.)"},
        {"id": "symbols", "name": "Symbols", "count": 182, "description": "Currency pairs, Crypto, Metals, Indices, OTC"},
        {"id": "timeframes", "name": "Timeframes", "count": 12, "description": "M1, M5, M15, H1, D1 and expiration times"},
        {"id": "targets", "name": "Price Targets", "count": 8, "description": "Entry, Stop Loss, Take Profit 1-5"},
        {"id": "gales", "name": "Martingale / Gales", "count": 4, "description": "Gale 1, Gale 2, No Gale"},
    ]


@router.get("/custom", summary="Get custom tokens")
async def get_custom_tokens():
    """Returns list of custom user-defined tokens."""
    return []
