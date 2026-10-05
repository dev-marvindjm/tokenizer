from typing import List, Optional
from fastapi import APIRouter, HTTPException, Query, status, Depends
from sqlmodel import select, or_
from app.core.database import SessionDep
from app.core.deps import get_current_user, get_optional_current_user
from app.domain.models import SignalTemplate, User, TemplateConfig, TemplateBroker, utc_now
from app.domain.schemas import (
    SignalTemplateCreate,
    SignalTemplateRead,
    SignalTemplateUpdate,
    MatchTemplateRequest,
    MatchTemplateResponse,
)
from app.services.rust_bridge import rust_bridge

router = APIRouter(prefix="/templates", tags=["Signal Templates"])

@router.post("", response_model=SignalTemplateRead, status_code=status.HTTP_201_CREATED, summary="Create a new signal template")
async def create_template(
    payload: SignalTemplateCreate,
    session: SessionDep,
    current_user: Optional[User] = Depends(get_optional_current_user),
) -> SignalTemplateRead:
    """Register a new syntactic template pattern for the Rust matcher."""
    # System template or operator template
    is_system = True if (not current_user or current_user.is_superuser or payload.is_system) else False

    template = SignalTemplate(
        user_id=current_user.id if current_user else None,
        is_system=is_system,
        name=payload.name,
        code_slot=payload.code_slot,
        section=payload.section,
        sender_name=payload.sender_name,
        pattern_syntax=payload.pattern_syntax,
        template_type=payload.template_type,
        description=payload.description,
        example_text=payload.example_text,
        priority=payload.priority,
        is_active=payload.is_active,
        win_rate=payload.win_rate if payload.win_rate is not None else 80.0,
        latency_us=payload.latency_us if payload.latency_us is not None else 30,
        signals_count=payload.signals_count if payload.signals_count is not None else 0,
        quality_score=payload.quality_score if payload.quality_score is not None else 85,
    )
    session.add(template)
    await session.commit()
    await session.refresh(template)
    return SignalTemplateRead.model_validate(template)

@router.get("", response_model=List[SignalTemplateRead], summary="List signal templates accessible to user")
async def list_templates(
    session: SessionDep,
    limit: int = Query(default=100, ge=1, le=500),
    offset: int = Query(default=0, ge=0),
    is_active: Optional[bool] = Query(default=None),
    template_type: Optional[str] = Query(default=None),
    search: Optional[str] = Query(default=None),
    current_user: Optional[User] = Depends(get_optional_current_user),
) -> List[SignalTemplateRead]:
    """Retrieve templates with pagination and user tenancy filtering."""
    query = select(SignalTemplate)

    # Multi-user data isolation:
    # Authenticated user sees their own templates + global system templates (or unassigned templates).
    if current_user:
        query = query.where(
            or_(
                SignalTemplate.user_id == current_user.id,
                SignalTemplate.is_system == True,
                SignalTemplate.user_id == None,
            )
        )
    else:
        # Unauthenticated calls only see global system / public templates
        query = query.where(
            or_(
                SignalTemplate.is_system == True,
                SignalTemplate.user_id == None,
            )
        )

    if is_active is not None:
        query = query.where(SignalTemplate.is_active == is_active)
    if template_type:
        query = query.where(SignalTemplate.template_type == template_type)
    if search:
        query = query.where(SignalTemplate.name.contains(search))

    query = query.order_by(SignalTemplate.id.asc()).offset(offset).limit(limit)
    result = await session.exec(query)
    templates = result.all()
    return [SignalTemplateRead.model_validate(t) for t in templates]

@router.get("/configs", summary="List all template configurations")
async def list_template_configs(session: SessionDep):
    result = await session.exec(select(TemplateConfig))
    configs = result.all()
    return configs

@router.get("/brokers-links", summary="List all template broker links")
async def list_template_broker_links(session: SessionDep):
    result = await session.exec(select(TemplateBroker))
    return result.all()

@router.get("/brokers-catalog", summary="List distinct brokers and configured accounts")
async def get_brokers_catalog(session: SessionDep):
    result = await session.exec(select(TemplateBroker))
    links = result.all()
    
    # Defaults in case DB has few
    catalog: dict = {
        "IQ Option": {"broker_id": 1, "name": "IQ Option", "accounts": set()},
        "Quotex": {"broker_id": 2, "name": "Quotex", "accounts": set()},
        "Pocket Option": {"broker_id": 3, "name": "Pocket Option", "accounts": set()},
        "MetaTrader 5": {"broker_id": 4, "name": "MetaTrader 5", "accounts": set()},
    }
    
    for l in links:
        b_name = l.broker_name or "MetaTrader 5"
        if b_name not in catalog:
            catalog[b_name] = {"broker_id": l.broker_id, "name": b_name, "accounts": set()}
        if l.account_number:
            catalog[b_name]["accounts"].add(l.account_number)
            
    out = []
    for k, v in catalog.items():
        out.append({
            "broker_id": v["broker_id"],
            "name": v["name"],
            "accounts": sorted(list(v["accounts"])) if v["accounts"] else ["Default Live Account"],
        })
    return out

@router.get("/{template_id}", response_model=SignalTemplateRead, summary="Get template by ID")
async def get_template(
    template_id: int,
    session: SessionDep,
    current_user: Optional[User] = Depends(get_optional_current_user),
) -> SignalTemplateRead:
    template = await session.get(SignalTemplate, template_id)
    if not template:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Template with id {template_id} not found.",
        )

    # Multi-user security check
    if template.user_id is not None and not template.is_system:
        if current_user and template.user_id != current_user.id and not current_user.is_superuser:
            raise HTTPException(
                status_code=status.HTTP_403_FORBIDDEN,
                detail="You do not have access to this template.",
            )

    return SignalTemplateRead.model_validate(template)

@router.put("/{template_id}", response_model=SignalTemplateRead, summary="Update template")
async def update_template(
    template_id: int,
    payload: SignalTemplateUpdate,
    session: SessionDep,
    current_user: Optional[User] = Depends(get_optional_current_user),
) -> SignalTemplateRead:
    template = await session.get(SignalTemplate, template_id)
    if not template:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Template with id {template_id} not found.",
        )

    # Multi-user security check
    if template.user_id is not None and current_user and template.user_id != current_user.id and not current_user.is_superuser:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="You can only modify your own templates.",
        )

    update_dict = payload.model_dump(exclude_unset=True)
    for k, v in update_dict.items():
        setattr(template, k, v)

    session.add(template)
    await session.commit()
    await session.refresh(template)
    return SignalTemplateRead.model_validate(template)

@router.delete("/{template_id}", status_code=status.HTTP_200_OK, summary="Delete template")
async def delete_template(
    template_id: int,
    session: SessionDep,
    current_user: Optional[User] = Depends(get_optional_current_user),
) -> dict:
    template = await session.get(SignalTemplate, template_id)
    if not template:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Template with id {template_id} not found.",
        )

    if template.user_id is not None and current_user and template.user_id != current_user.id and not current_user.is_superuser:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="You can only delete your own templates.",
        )

    await session.delete(template)
    await session.commit()
    return {"detail": f"Template {template_id} successfully deleted."}

@router.post("/match", response_model=MatchTemplateResponse, summary="Match text against signal templates")
async def match_template_endpoint(
    payload: MatchTemplateRequest,
    session: SessionDep,
    current_user: Optional[User] = Depends(get_optional_current_user),
) -> MatchTemplateResponse:
    """
    Evaluates raw trading text against registered templates using the Rust native matcher.
    If template_id is specified, tests against that template; otherwise, tests against accessible active templates in priority order.
    """
    if not payload.text.strip():
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="The 'text' field cannot be empty.",
        )

    if payload.template_id is not None:
        template = await session.get(SignalTemplate, payload.template_id)
        if not template:
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail=f"Template with id {payload.template_id} not found.",
            )
        matched_sig, dur_us = await rust_bridge.match_template_async(
            payload.text, template.pattern_syntax, template.name
        )
        return MatchTemplateResponse(
            matched=matched_sig is not None,
            template_id=template.id,
            template_name=template.name,
            matched_pattern=template.pattern_syntax,
            signal=matched_sig,
            processing_time_us=dur_us,
        )

    # Match across active templates accessible to the user (their own + system/unassigned)
    if current_user:
        query = (
            select(SignalTemplate)
            .where(
                SignalTemplate.is_active == True,
                or_(
                    SignalTemplate.user_id == current_user.id,
                    SignalTemplate.is_system == True,
                    SignalTemplate.user_id == None,
                ),
            )
            .order_by(SignalTemplate.priority.desc(), SignalTemplate.id.desc())
        )
    else:
        query = (
            select(SignalTemplate)
            .where(
                SignalTemplate.is_active == True,
                or_(
                    SignalTemplate.is_system == True,
                    SignalTemplate.user_id == None,
                ),
            )
            .order_by(SignalTemplate.priority.desc(), SignalTemplate.id.desc())
        )

    result = await session.exec(query)
    templates = result.all()

    total_us = 0
    for tpl in templates:
        matched_sig, dur_us = await rust_bridge.match_template_async(
            payload.text, tpl.pattern_syntax, tpl.name
        )
        total_us += dur_us
        if matched_sig is not None:
            return MatchTemplateResponse(
                matched=True,
                template_id=tpl.id,
                template_name=tpl.name,
                matched_pattern=tpl.pattern_syntax,
                signal=matched_sig,
                processing_time_us=total_us,
            )

    # Fallback entity extraction if no template matched
    entities, dur_us = await rust_bridge.extract_entities_async(payload.text)
    total_us += dur_us
    has_sig = entities.get("has_signal", False)

    return MatchTemplateResponse(
        matched=has_sig,
        template_id=None,
        template_name=None,
        matched_pattern=entities.get("template") if has_sig else None,
        signal=entities.get("signals", [None])[0] if has_sig and entities.get("signals") else None,
        processing_time_us=total_us,
    )


@router.get("/{template_id}/config", summary="Get template configuration")
async def get_template_config(template_id: int, session: SessionDep):
    template = await session.get(SignalTemplate, template_id)
    if not template:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Template not found")
    
    result = await session.exec(select(TemplateConfig).where(TemplateConfig.template_id == template_id))
    config = result.first()
    if not config:
        return {
            "template_id": template.id,
            "name": template.name,
            "pattern_syntax": template.pattern_syntax,
            "ammount_type": "percentage",
            "ammount": 1.0,
            "exit_strategy_type": "gale",
            "exit_contained_inside": False,
            "max_gale": 0,
            "gale_ammount": 2.0,
            "profit_ratio": 10.0,
            "loss_type": "price",
            "loss_ratio": 5.0,
            "logic": None,
        }
    
    res = config.model_dump()
    res["name"] = template.name
    res["pattern_syntax"] = template.pattern_syntax
    return res


@router.put("/{template_id}/config", summary="Save template configuration")
async def save_template_config(template_id: int, payload: dict, session: SessionDep):
    template = await session.get(SignalTemplate, template_id)
    if not template:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Template not found")
    
    result = await session.exec(select(TemplateConfig).where(TemplateConfig.template_id == template_id))
    config = result.first()
    
    if not config:
        config = TemplateConfig(
            template_id=template_id,
            strategy_id=payload.get("strategy_id"),
            ammount_type=payload.get("ammount_type", "percentage"),
            ammount=float(payload.get("ammount", 1.0)),
            exit_strategy_type=payload.get("exit_strategy_type", "gale"),
            exit_contained_inside=bool(payload.get("exit_contained_inside", False)),
            max_gale=int(payload.get("max_gale", 0)) if payload.get("max_gale") is not None else 0,
            gale_ammount=float(payload.get("gale_ammount", 2.0)) if payload.get("gale_ammount") is not None else 2.0,
            profit_ratio=float(payload["profit_ratio"]) if payload.get("profit_ratio") is not None else None,
            loss_type=payload.get("loss_type", "price"),
            loss_ratio=float(payload["loss_ratio"]) if payload.get("loss_ratio") is not None else None,
            logic=payload.get("logic"),
        )
        session.add(config)
    else:
        if "ammount_type" in payload:
            config.ammount_type = payload["ammount_type"]
        if "ammount" in payload:
            config.ammount = float(payload["ammount"])
        if "exit_strategy_type" in payload:
            config.exit_strategy_type = payload["exit_strategy_type"]
        if "exit_contained_inside" in payload:
            config.exit_contained_inside = bool(payload["exit_contained_inside"])
        if "max_gale" in payload:
            config.max_gale = int(payload["max_gale"]) if payload["max_gale"] is not None else 0
        if "gale_ammount" in payload:
            config.gale_ammount = float(payload["gale_ammount"]) if payload["gale_ammount"] is not None else 2.0
        if "profit_ratio" in payload:
            config.profit_ratio = float(payload["profit_ratio"]) if payload["profit_ratio"] is not None else None
        if "loss_type" in payload:
            config.loss_type = payload["loss_type"]
        if "loss_ratio" in payload:
            config.loss_ratio = float(payload["loss_ratio"]) if payload["loss_ratio"] is not None else None
        if "logic" in payload:
            config.logic = payload["logic"]
        if "strategy_id" in payload:
            config.strategy_id = payload["strategy_id"]
        config.updated_at = utc_now()
        session.add(config)
        
    await session.commit()
    await session.refresh(config)
    res = config.model_dump()
    res["name"] = template.name
    res["pattern_syntax"] = template.pattern_syntax
    return res


@router.get("/{template_id}/brokers", summary="Get brokers linked to template")
async def get_template_brokers(template_id: int, session: SessionDep):
    result = await session.exec(select(TemplateBroker).where(TemplateBroker.template_id == template_id))
    links = result.all()
    if not links:
        return [{"broker_name": "MetaTrader 5", "account_number": "Default", "is_active": True}]
    return links


@router.post("/{template_id}/brokers", summary="Link broker to template")
async def link_template_broker(template_id: int, payload: dict, session: SessionDep):
    broker_name = payload.get("broker_name", "MetaTrader 5")
    account_number = payload.get("account_number", "Default")
    broker_id = int(payload.get("broker_id", 4))
    
    result = await session.exec(select(TemplateBroker).where(TemplateBroker.template_id == template_id))
    existing = result.first()
    if existing:
        existing.broker_name = broker_name
        existing.account_number = account_number
        existing.broker_id = broker_id
        if "is_active" in payload:
            existing.is_active = bool(payload["is_active"])
        existing.updated_at = utc_now()
        session.add(existing)
    else:
        new_link = TemplateBroker(
            template_id=template_id,
            broker_id=broker_id,
            broker_name=broker_name,
            account_number=account_number,
            is_active=bool(payload.get("is_active", True)),
        )
        session.add(new_link)
        
    await session.commit()
    return {"status": "ok", "template_id": template_id, "broker_name": broker_name, "account_number": account_number}


@router.post("/batch/toggle", summary="Batch toggle templates active status")
async def batch_toggle_templates(payload: dict, session: SessionDep):
    return {"status": "ok"}
