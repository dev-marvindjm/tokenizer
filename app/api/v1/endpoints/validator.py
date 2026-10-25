import json
import logging
import sqlite3
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import List, Dict, Any, Optional
from fastapi import APIRouter, HTTPException, Query, Response, status
from pydantic import BaseModel, Field

from app.core.config import settings
from app.services.rust_bridge import rust_bridge

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/validator", tags=["Validator & Ground Truth Studio"])

def get_db_path() -> Path:
    p = settings.CONSOLIDATED_DB_PATH
    if not p.is_absolute():
        p = Path.cwd() / p
    if not p.exists():
        # Fallback to checking in data dir
        alt = Path("/Users/macseqoia/gits/tokenizer/data/consolidated_signals.db")
        if alt.exists():
            return alt
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Consolidated database not found at {p}. Please run scripts/consolidate_databases.py first."
        )
    return p


def get_db_connection(row_factory: bool = False) -> sqlite3.Connection:
    """Opens a SQLite connection with WAL mode and FULL synchronous durability to prevent corruption on power loss."""
    db_path = get_db_path()
    conn = sqlite3.connect(db_path, timeout=30.0)
    conn.execute("PRAGMA journal_mode=WAL;")
    conn.execute("PRAGMA synchronous=FULL;")
    conn.execute("PRAGMA foreign_keys=ON;")
    if row_factory:
        conn.row_factory = sqlite3.Row
    return conn

# --- Pydantic Schemas ---

class MessageItem(BaseModel):
    id: int
    source_db: str
    original_id: Optional[int] = None
    sender_id: Optional[int] = None
    sender_name: Optional[str] = None
    channel_handle: Optional[str] = None
    external_message_id: Optional[str] = None
    raw_text: str
    published_at: Optional[str] = None
    captured_at: Optional[str] = None
    processed_status: Optional[str] = None
    is_validated: bool = False
    is_signal: Optional[bool] = None
    signals_count: Optional[int] = None
    ground_truth_json: Optional[str] = None

class MessageListResponse(BaseModel):
    items: List[MessageItem]
    total: int
    page: int
    page_size: int
    total_pages: int

class TemplateItem(BaseModel):
    id: int
    name: str
    pattern_syntax: str
    template_type: str
    description: Optional[str] = None
    example_text: Optional[str] = None
    priority: int
    is_active: bool
    sender_name: Optional[str] = None

class SenderItem(BaseModel):
    id: int
    display_name: str
    handle: Optional[str] = None
    sender_type: Optional[str] = None
    messages_count: int

class StatsResponse(BaseModel):
    total_messages: int
    total_templates: int
    total_senders: int
    total_validated: int
    signals_count: int
    non_signals_count: int
    telegram_messages: Optional[int] = None
    telegram_senders: Optional[int] = None

class ParseRequest(BaseModel):
    text: str
    template_id: Optional[int] = None
    pattern_syntax: Optional[str] = None
    template_name: Optional[str] = None
    end_char_new_line: Optional[bool] = None
    sender_name: Optional[str] = None

class TemplateCreateRequest(BaseModel):
    name: Optional[str] = None
    pattern_syntax: str
    sender_name: Optional[str] = None
    template_type: str = "market"
    description: Optional[str] = None
    example_text: Optional[str] = None
    priority: int = 100
    is_active: bool = True

class TemplateUpdateRequest(BaseModel):
    name: Optional[str] = None
    pattern_syntax: Optional[str] = None
    sender_name: Optional[str] = None
    template_type: Optional[str] = None
    description: Optional[str] = None
    example_text: Optional[str] = None
    priority: Optional[int] = None
    is_active: Optional[bool] = None

class ParseResponse(BaseModel):
    parse_template_json: Optional[str] = None
    parse_template_us: int = 0
    parse_signal_json: Optional[str] = None
    parse_signal_us: int = 0
    matched_template_id: Optional[int] = None
    matched_template_name: Optional[str] = None
    matched_pattern: Optional[str] = None
    matched_sender_name: Optional[str] = None
    match_score: Optional[float] = None
    detected_signal_count: int = 0
    exact_match: bool = False
    exact_signals_count: int = 0
    exact_signals_json: Optional[str] = None
    exact_latency_us: int = 0
    templates_tested_count: int = 0

class TestExactRequest(BaseModel):
    text: str
    pattern_syntax: str
    end_char_new_line: Optional[bool] = None

class TestExactResponse(BaseModel):
    is_match: bool
    signals_count: int
    signals_json: Optional[str] = None
    latency_us: int = 0

class BatchItemResult(BaseModel):
    message_id: int
    sender_name: Optional[str] = None
    raw_text_preview: str
    is_match: bool
    signals_count: int
    latency_us: int
    signals_json: Optional[str] = None
    matched_template_id: Optional[int] = None
    matched_template_name: Optional[str] = None

class SenderBatchStats(BaseModel):
    sender_name: str
    tested: int = 0
    matched: int = 0
    failed: int = 0
    signals_count: int = 0

class TestBatchRequest(BaseModel):
    pattern_syntax: Optional[str] = ""
    mode: str = "at_least_one"  # "at_least_one", "all", "selected"
    sender_names: Optional[List[str]] = None
    message_ids: Optional[List[int]] = None
    end_char_new_line: Optional[bool] = None
    limit: Optional[int] = 500
    iterate_channel_templates: bool = True

class TestBatchResponse(BaseModel):
    mode: str
    pattern_syntax: str
    total_tested: int
    matched_count: int
    failed_count: int
    signals_total_count: int
    match_rate_pct: float
    avg_latency_us: float
    summary_by_sender: Dict[str, SenderBatchStats]
    results: List[BatchItemResult]

class ValidateRequest(BaseModel):
    message_id: int
    is_signal: bool
    signals_count: int = 0
    ground_truth_json: str
    parse_template_json: Optional[str] = None
    parse_signal_json: Optional[str] = None
    template_id: Optional[int] = None
    template_name: Optional[str] = None
    notes: Optional[str] = None

class BatchValidateRequest(BaseModel):
    message_ids: List[int]
    is_signal: bool = False
    signals_count: int = 0
    ground_truth_json: str = "[]"
    notes: Optional[str] = "Batch marked non-signal"

# --- Endpoints ---

@router.get("/stats", response_model=StatsResponse)
async def get_validator_stats():
    """Returns database summary statistics for the validator dashboard."""
    conn = get_db_connection()
    cur = conn.cursor()

    total_msgs = cur.execute("SELECT count(*) FROM messages").fetchone()[0]
    total_tpls = cur.execute("SELECT count(*) FROM templates").fetchone()[0]
    total_snds = cur.execute("SELECT count(*) FROM senders").fetchone()[0]
    total_val = cur.execute("SELECT count(*) FROM messages WHERE is_validated = 1").fetchone()[0]
    sig_count = cur.execute("SELECT count(*) FROM messages WHERE is_validated = 1 AND is_signal = 1").fetchone()[0]
    non_sig_count = cur.execute("SELECT count(*) FROM messages WHERE is_validated = 1 AND is_signal = 0").fetchone()[0]

    tg_msgs = cur.execute("SELECT count(*) FROM messages WHERE sender_name IS NULL OR LOWER(sender_name) NOT LIKE 'wa%'").fetchone()[0]
    tg_snds = cur.execute("SELECT count(DISTINCT sender_name) FROM messages WHERE sender_name IS NOT NULL AND LOWER(sender_name) NOT LIKE 'wa%'").fetchone()[0]

    conn.close()
    return StatsResponse(
        total_messages=total_msgs,
        total_templates=total_tpls,
        total_senders=total_snds,
        total_validated=total_val,
        signals_count=sig_count,
        non_signals_count=non_sig_count,
        telegram_messages=tg_msgs,
        telegram_senders=tg_snds,
    )

@router.get("/messages", response_model=MessageListResponse)
async def get_messages(
    page: int = Query(default=1, ge=1),
    page_size: int = Query(default=50, ge=1, le=200),
    sender_name: Optional[str] = None,
    sender_names: Optional[List[str]] = Query(default=None),
    status_filter: str = Query(default="all", alias="status"),
    search: Optional[str] = None,
    include_wa: bool = Query(default=False),
):
    """
    Paginated list of messages from consolidated_signals.db with filtering.
    By default excludes WhatsApp senders (wa*), focusing only on Telegram channels.
    status_filter options: 'all', 'pending', 'validated', 'signal', 'non_signal'
    """
    conn = get_db_connection(row_factory=True)
    cur = conn.cursor()

    conditions = []
    params = []

    # Exclude WhatsApp senders unless explicitly asked
    if not include_wa:
        conditions.append("(m.sender_name IS NULL OR LOWER(m.sender_name) NOT LIKE 'wa%')")

    if sender_names and len(sender_names) > 0:
        placeholders = ",".join("?" for _ in sender_names)
        conditions.append(f"m.sender_name IN ({placeholders})")
        params.extend(sender_names)
    elif sender_name:
        conditions.append("m.sender_name = ?")
        params.append(sender_name)

    if status_filter == "pending":
        conditions.append("m.is_validated = 0")
    elif status_filter == "validated":
        conditions.append("m.is_validated = 1")
    elif status_filter == "signal":
        conditions.append("m.is_validated = 1 AND m.is_signal = 1")
    elif status_filter == "non_signal":
        conditions.append("m.is_validated = 1 AND m.is_signal = 0")

    if search and search.strip():
        conditions.append("m.raw_text LIKE ?")
        params.append(f"%{search.strip()}%")

    where_clause = " WHERE " + " AND ".join(conditions) if conditions else ""

    # Total count
    count_sql = f"SELECT count(*) FROM messages m {where_clause}"
    total = cur.execute(count_sql, params).fetchone()[0]

    offset = (page - 1) * page_size
    query_sql = f"""
        SELECT m.id, m.source_db, m.original_id, m.sender_id, m.sender_name,
               m.channel_handle, m.external_message_id, m.raw_text, m.published_at,
               m.captured_at, m.processed_status, m.is_validated,
               COALESCE(v.is_signal, m.is_signal) AS is_signal,
               v.signals_count, v.ground_truth_json
        FROM messages m
        LEFT JOIN validated_test_cases v ON m.id = v.message_id
        {where_clause}
        ORDER BY m.id ASC
        LIMIT ? OFFSET ?
    """
    rows = cur.execute(query_sql, params + [page_size, offset]).fetchall()

    items = [
        MessageItem(
            id=r["id"],
            source_db=r["source_db"],
            original_id=r["original_id"],
            sender_id=r["sender_id"],
            sender_name=r["sender_name"],
            channel_handle=r["channel_handle"],
            external_message_id=r["external_message_id"],
            raw_text=r["raw_text"],
            published_at=r["published_at"],
            captured_at=r["captured_at"],
            processed_status=r["processed_status"],
            is_validated=bool(r["is_validated"]),
            is_signal=bool(r["is_signal"]) if r["is_signal"] is not None else None,
            signals_count=r["signals_count"],
            ground_truth_json=r["ground_truth_json"],
        )
        for r in rows
    ]

    total_pages = (total + page_size - 1) // page_size if total > 0 else 1
    conn.close()

    return MessageListResponse(
        items=items,
        total=total,
        page=page,
        page_size=page_size,
        total_pages=total_pages,
    )

@router.get("/messages/{message_id}", response_model=MessageItem)
async def get_message_detail(message_id: int):
    """Fetch complete detail for a single message including ground truth if validated."""
    conn = get_db_connection(row_factory=True)
    cur = conn.cursor()

    row = cur.execute("""
        SELECT m.id, m.source_db, m.original_id, m.sender_id, m.sender_name,
               m.channel_handle, m.external_message_id, m.raw_text, m.published_at,
               m.captured_at, m.processed_status, m.is_validated,
               COALESCE(v.is_signal, m.is_signal) AS is_signal,
               v.signals_count, v.ground_truth_json
        FROM messages m
        LEFT JOIN validated_test_cases v ON m.id = v.message_id
        WHERE m.id = ?
    """, (message_id,)).fetchone()

    conn.close()
    if not row:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Message not found")

    return MessageItem(
        id=row["id"],
        source_db=row["source_db"],
        original_id=row["original_id"],
        sender_id=row["sender_id"],
        sender_name=row["sender_name"],
        channel_handle=row["channel_handle"],
        external_message_id=row["external_message_id"],
        raw_text=row["raw_text"],
        published_at=row["published_at"],
        captured_at=row["captured_at"],
        processed_status=row["processed_status"],
        is_validated=bool(row["is_validated"]),
        is_signal=bool(row["is_signal"]) if row["is_signal"] is not None else None,
        signals_count=row["signals_count"],
        ground_truth_json=row["ground_truth_json"],
    )

@router.get("/templates", response_model=List[TemplateItem])
async def get_templates(
    sender_name: Optional[str] = Query(default=None),
    include_global: bool = Query(default=False),
):
    """
    Returns available templates for pattern testing.
    If sender_name is provided:
      - If include_global is False, returns templates specifically linked to that sender.
      - If include_global is True, returns sender's templates + unassigned/global templates.
    """
    conn = get_db_connection(row_factory=True)
    cur = conn.cursor()

    if sender_name and sender_name.strip() and sender_name.strip().lower() != "all":
        s_clean = sender_name.strip()
        if include_global:
            rows = cur.execute("""
                SELECT id, name, pattern_syntax, template_type, description,
                       example_text, priority, is_active, sender_name
                FROM templates
                WHERE sender_name = ? OR sender_name IS NULL OR sender_name = ''
                ORDER BY CASE WHEN sender_name = ? THEN 0 ELSE 1 END, priority DESC, id DESC
            """, (s_clean, s_clean)).fetchall()
        else:
            rows = cur.execute("""
                SELECT id, name, pattern_syntax, template_type, description,
                       example_text, priority, is_active, sender_name
                FROM templates
                WHERE sender_name = ?
                ORDER BY priority DESC, id DESC
            """, (s_clean,)).fetchall()
    else:
        rows = cur.execute("""
            SELECT id, name, pattern_syntax, template_type, description,
                   example_text, priority, is_active, sender_name
            FROM templates
            ORDER BY priority DESC, id DESC
        """).fetchall()

    conn.close()
    return [
        TemplateItem(
            id=r["id"],
            name=r["name"],
            pattern_syntax=r["pattern_syntax"],
            template_type=r["template_type"],
            description=r["description"],
            example_text=r["example_text"],
            priority=r["priority"],
            is_active=bool(r["is_active"]),
            sender_name=r["sender_name"],
        )
        for r in rows
    ]

@router.post("/templates", response_model=TemplateItem, status_code=status.HTTP_201_CREATED)
async def create_template(payload: TemplateCreateRequest):
    """Creates a new template linked to a sender and saves it to consolidated_signals.db."""
    clean_pattern = payload.pattern_syntax.strip()
    if not clean_pattern:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="La sintaxis del template (pattern_syntax) no puede estar vacía."
        )

    clean_name = payload.name.strip() if payload.name and payload.name.strip() else ""
    clean_sender = payload.sender_name.strip() if payload.sender_name and payload.sender_name.strip() else None
    if not clean_name:
        clean_name = f"Template — {clean_sender or 'General'}"

    conn = get_db_connection(row_factory=True)
    cur = conn.cursor()

    cur.execute("""
        INSERT INTO templates (
            source_db, original_id, name, pattern_syntax, template_type,
            description, example_text, priority, is_active, sender_name
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, (
        "consolidated_signals.db",
        None,
        clean_name,
        clean_pattern,
        payload.template_type or "market",
        payload.description,
        payload.example_text,
        payload.priority,
        1 if payload.is_active else 0,
        clean_sender,
    ))
    conn.commit()
    new_id = cur.lastrowid

    row = cur.execute("""
        SELECT id, name, pattern_syntax, template_type, description,
               example_text, priority, is_active, sender_name
        FROM templates WHERE id = ?
    """, (new_id,)).fetchone()
    conn.close()

    # Optional mirror into trading_signals_app.db if available
    try:
        app_db = settings.DATA_DIR / "trading_signals_app.db"
        if app_db.exists():
            now_iso = datetime.now(timezone.utc).isoformat()
            app_conn = sqlite3.connect(app_db)
            app_cur = app_conn.cursor()
            app_cur.execute("""
                INSERT INTO signal_templates (
                    name, pattern_syntax, template_type, description,
                    example_text, priority, is_active, sender_name,
                    is_system, win_rate, latency_us, signals_count, quality_score,
                    created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, 80.0, 30, 0, 85, ?, ?)
            """, (
                clean_name,
                clean_pattern,
                payload.template_type or "market",
                payload.description,
                payload.example_text,
                payload.priority,
                1 if payload.is_active else 0,
                clean_sender,
                now_iso,
                now_iso,
            ))
            app_conn.commit()
            app_conn.close()
    except Exception as e:
        logger.warning(f"Could not mirror template into trading_signals_app.db: {e}")

    return TemplateItem(
        id=row["id"],
        name=row["name"],
        pattern_syntax=row["pattern_syntax"],
        template_type=row["template_type"],
        description=row["description"],
        example_text=row["example_text"],
        priority=row["priority"],
        is_active=bool(row["is_active"]),
        sender_name=row["sender_name"],
    )

@router.put("/templates/{template_id}", response_model=TemplateItem)
async def update_template(template_id: int, payload: TemplateUpdateRequest):
    """Updates an existing template by ID in consolidated_signals.db."""
    conn = get_db_connection(row_factory=True)
    cur = conn.cursor()

    existing = cur.execute("""
        SELECT id, name, pattern_syntax, template_type, description,
               example_text, priority, is_active, sender_name
        FROM templates WHERE id = ?
    """, (template_id,)).fetchone()
    if not existing:
        conn.close()
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Template no encontrado")

    new_pattern = payload.pattern_syntax.strip() if payload.pattern_syntax is not None else existing["pattern_syntax"]
    if not new_pattern:
        conn.close()
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="La sintaxis del template (pattern_syntax) no puede estar vacía."
        )

    new_name = payload.name.strip() if payload.name is not None and payload.name.strip() else existing["name"]
    new_sender = (payload.sender_name.strip() if payload.sender_name.strip() else None) if payload.sender_name is not None else existing["sender_name"]
    new_type = payload.template_type if payload.template_type is not None else existing["template_type"]
    new_desc = payload.description if payload.description is not None else existing["description"]
    new_example = payload.example_text if payload.example_text is not None else existing["example_text"]
    new_prio = payload.priority if payload.priority is not None else existing["priority"]
    new_active = (1 if payload.is_active else 0) if payload.is_active is not None else existing["is_active"]

    cur.execute("""
        UPDATE templates
        SET name = ?,
            pattern_syntax = ?,
            sender_name = ?,
            template_type = ?,
            description = ?,
            example_text = ?,
            priority = ?,
            is_active = ?
        WHERE id = ?
    """, (
        new_name,
        new_pattern,
        new_sender,
        new_type,
        new_desc,
        new_example,
        new_prio,
        new_active,
        template_id,
    ))
    conn.commit()

    row = cur.execute("""
        SELECT id, name, pattern_syntax, template_type, description,
               example_text, priority, is_active, sender_name
        FROM templates WHERE id = ?
    """, (template_id,)).fetchone()
    conn.close()

    return TemplateItem(
        id=row["id"],
        name=row["name"],
        pattern_syntax=row["pattern_syntax"],
        template_type=row["template_type"],
        description=row["description"],
        example_text=row["example_text"],
        priority=row["priority"],
        is_active=bool(row["is_active"]),
        sender_name=row["sender_name"],
    )

@router.delete("/templates/{template_id}")
async def delete_template(template_id: int):
    """Deletes a template by ID from consolidated_signals.db."""
    conn = get_db_connection()
    cur = conn.cursor()
    cur.execute("DELETE FROM templates WHERE id = ?", (template_id,))
    deleted = cur.rowcount
    conn.commit()
    conn.close()
    if deleted == 0:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Template no encontrado")
    return {"status": "success", "deleted_id": template_id}

@router.get("/senders", response_model=List[SenderItem])
async def get_senders(include_wa: bool = Query(default=False)):
    """
    Returns list of distinct Telegram senders/channels ordered by message count.
    WhatsApp senders (wa*) are omitted by default.
    """
    conn = get_db_connection(row_factory=True)
    cur = conn.cursor()

    condition = "" if include_wa else "AND LOWER(sender_name) NOT LIKE 'wa%'"
    query = f"""
        SELECT sender_name, channel_handle, count(*) as count
        FROM messages
        WHERE sender_name IS NOT NULL AND sender_name != '' {condition}
        GROUP BY sender_name
        ORDER BY count DESC
    """
    rows = cur.execute(query).fetchall()

    conn.close()
    return [
        SenderItem(
            id=idx + 1,
            display_name=r["sender_name"],
            handle=r["channel_handle"],
            messages_count=r["count"],
        )
        for idx, r in enumerate(rows)
    ]

@router.post("/parse", response_model=ParseResponse)
async def parse_message_endpoint(req: ParseRequest):
    """
    Executes both parse_signal and parse_template on the given text.
    If no template is specified, automatically iterates over ALL templates in the database
    (including newly created templates, prioritizing templates for req.sender_name first)
    to verify if any template matches via strict 100% parse_template_exact (with fallback to scored match).
    """
    text = req.text.strip()
    if not text:
        return ParseResponse()

    # 1. Execute parse_signal_json
    sig_json, sig_us = await rust_bridge.parse_signal_async(text)

    # Count detected signals from sig_json
    det_count = 0
    if sig_json:
        try:
            parsed_obj = json.loads(sig_json)
            if isinstance(parsed_obj, list):
                det_count = len(parsed_obj)
            elif isinstance(parsed_obj, dict):
                signals_arr = parsed_obj.get("signals", [])
                det_count = len(signals_arr)
        except Exception:
            det_count = 0

    # 2. Execute parse_template_exact and parse_template_json
    tpl_json = None
    tpl_us = 0
    matched_id = req.template_id
    matched_name = req.template_name
    matched_sender_name = None
    match_score = None
    resolved_pattern = req.pattern_syntax
    exact_json = None
    exact_count = 0
    exact_us = 0
    templates_tested_count = 0

    conn = get_db_connection(row_factory=True)
    cur = conn.cursor()

    if req.pattern_syntax:
        # Pattern provided directly
        templates_tested_count = 1
        tpl_json, tpl_us = await rust_bridge.parse_template_async(
            text, req.pattern_syntax, req.template_name
        )
        exact_json, exact_count, exact_us = await rust_bridge.parse_template_exact_async(
            text, req.pattern_syntax, req.end_char_new_line
        )
    elif req.template_id:
        # Template specified by ID
        row = cur.execute(
            "SELECT id, name, pattern_syntax, sender_name FROM templates WHERE id = ?",
            (req.template_id,)
        ).fetchone()
        conn.close()
        if row:
            templates_tested_count = 1
            matched_name = row["name"]
            matched_sender_name = row["sender_name"]
            resolved_pattern = row["pattern_syntax"]
            tpl_json, tpl_us = await rust_bridge.parse_template_async(
                text, row["pattern_syntax"], matched_name
            )
            exact_json, exact_count, exact_us = await rust_bridge.parse_template_exact_async(
                text, row["pattern_syntax"], req.end_char_new_line
            )
    else:
        # Auto-match: iterate through ALL templates in the database (including newly created templates)
        if req.sender_name and req.sender_name.strip():
            s_clean = req.sender_name.strip()
            all_tpls = cur.execute(
                """SELECT id, name, pattern_syntax, sender_name FROM templates 
                   ORDER BY CASE 
                       WHEN sender_name = ? THEN 0 
                       WHEN sender_name IS NULL OR sender_name = '' THEN 1 
                       ELSE 2 
                   END, is_active DESC, priority DESC, id DESC""",
                (s_clean,)
            ).fetchall()
        else:
            all_tpls = cur.execute(
                "SELECT id, name, pattern_syntax, sender_name FROM templates ORDER BY is_active DESC, priority DESC, id DESC"
            ).fetchall()
        conn.close()

        templates_tested_count = len(all_tpls)
        tpl_dicts = [dict(r) for r in all_tpls]

        def _iterate_all_templates() -> Dict[str, Any]:
            total_exact_time = 0
            # Pass 1: Strict 100% parse_template verification across all templates
            for tpl in tpl_dicts:
                pat = (tpl["pattern_syntax"] or "").strip()
                if not pat:
                    continue
                e_json, e_cnt, e_us = rust_bridge.parse_template_exact_sync(
                    text, pat, req.end_char_new_line
                )
                total_exact_time += e_us
                if e_json is not None and e_cnt > 0:
                    return {
                        "exact_match": True,
                        "exact_json": e_json,
                        "exact_count": e_cnt,
                        "exact_us": e_us,
                        "tpl_json": e_json,
                        "tpl_us": total_exact_time,
                        "matched_id": tpl["id"],
                        "matched_name": tpl["name"],
                        "matched_pattern": tpl["pattern_syntax"],
                        "matched_sender_name": tpl["sender_name"],
                        "match_score": 100.0,
                    }

            # Pass 2: Fallback scored match if no strict 100% match was found
            best_score = -1.0
            best_json = None
            best_id = None
            best_name = None
            best_pat = None
            best_sender = None
            best_time = 0

            for tpl in tpl_dicts:
                pat = (tpl["pattern_syntax"] or "").strip()
                if not pat:
                    continue
                res_str, d_us = rust_bridge.parse_template_sync(text, pat, tpl["name"])
                if res_str:
                    try:
                        obj = json.loads(res_str)
                        sc = float(obj.get("match_score", 0.0))
                        if sc > best_score:
                            best_score = sc
                            best_json = res_str
                            best_id = tpl["id"]
                            best_name = tpl["name"]
                            best_pat = tpl["pattern_syntax"]
                            best_sender = tpl["sender_name"]
                            best_time = d_us
                            if best_score >= 100.0:
                                break
                    except Exception:
                        pass

            return {
                "exact_match": False,
                "exact_json": None,
                "exact_count": 0,
                "exact_us": total_exact_time,
                "tpl_json": best_json,
                "tpl_us": best_time or total_exact_time,
                "matched_id": best_id,
                "matched_name": best_name,
                "matched_pattern": best_pat,
                "matched_sender_name": best_sender,
                "match_score": best_score if best_json is not None else None,
            }

        import asyncio
        iter_res = await asyncio.to_thread(_iterate_all_templates)
        exact_json = iter_res["exact_json"]
        exact_count = iter_res["exact_count"]
        exact_us = iter_res["exact_us"]
        tpl_json = iter_res["tpl_json"]
        tpl_us = iter_res["tpl_us"]
        matched_id = iter_res["matched_id"]
        matched_name = iter_res["matched_name"]
        resolved_pattern = iter_res["matched_pattern"]
        matched_sender_name = iter_res["matched_sender_name"]
        match_score = iter_res["match_score"]

    if req.pattern_syntax:
        conn.close()

    # Extract score if tpl_json present and match_score not already extracted
    if tpl_json and match_score is None:
        try:
            obj = json.loads(tpl_json)
            if "match_score" in obj:
                match_score = float(obj.get("match_score", 0.0))
            elif obj.get("is_match"):
                match_score = 100.0
        except Exception:
            pass

    return ParseResponse(
        parse_template_json=tpl_json,
        parse_template_us=tpl_us,
        parse_signal_json=sig_json,
        parse_signal_us=sig_us,
        matched_template_id=matched_id,
        matched_template_name=matched_name,
        matched_pattern=resolved_pattern,
        matched_sender_name=matched_sender_name,
        match_score=match_score,
        detected_signal_count=det_count,
        exact_match=(exact_json is not None),
        exact_signals_count=exact_count,
        exact_signals_json=exact_json,
        exact_latency_us=exact_us,
        templates_tested_count=templates_tested_count,
    )

@router.post("/test-exact", response_model=TestExactResponse)
async def test_template_exact_endpoint(req: TestExactRequest):
    """
    Directly tests the strict parse_template function against text and editable pattern syntax.
    Returns whether it matched (is_match), the count of extracted signals, the full JSON and latency.
    """
    text = req.text.strip()
    pattern = req.pattern_syntax.strip()
    if not text or not pattern:
        return TestExactResponse(is_match=False, signals_count=0, signals_json=None, latency_us=0)

    json_str, count, us = await rust_bridge.parse_template_exact_async(
        text, pattern, req.end_char_new_line
    )
    return TestExactResponse(
        is_match=(json_str is not None),
        signals_count=count,
        signals_json=json_str,
        latency_us=us,
    )

@router.post("/test-batch", response_model=TestBatchResponse)
async def test_template_batch_endpoint(req: TestBatchRequest):
    """
    Executes parse_template globally across multiple Telegram senders.
    Iterates through all templates registered for each message's channel (plus global/custom templates)
    so that channels with multiple templates are accurately verified.
    Modes:
      - 'at_least_one': 1 message per selected/all Telegram channel.
      - 'all': All messages across selected/all Telegram channels (up to limit).
      - 'selected': Only the specifically checked message IDs.
    """
    pattern = (req.pattern_syntax or "").strip()

    conn = get_db_connection(row_factory=True)
    cur = conn.cursor()

    # Load all templates ordered by active status, priority, and ID
    all_tpl_rows = cur.execute(
        "SELECT id, name, pattern_syntax, sender_name FROM templates ORDER BY is_active DESC, priority DESC, id DESC"
    ).fetchall()
    all_templates = [dict(r) for r in all_tpl_rows]

    if not pattern and not all_templates:
        conn.close()
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="No hay templates disponibles para evaluar.",
        )

    target_msgs: List[sqlite3.Row] = []

    if req.mode == "selected":
        if req.message_ids and len(req.message_ids) > 0:
            placeholders = ",".join("?" for _ in req.message_ids)
            query = f"""
                SELECT id, sender_name, raw_text
                FROM messages
                WHERE id IN ({placeholders})
                ORDER BY id ASC
            """
            target_msgs = cur.execute(query, req.message_ids).fetchall()
    elif req.mode == "at_least_one":
        # Target channels: specific selected senders or all Telegram senders
        if req.sender_names and len(req.sender_names) > 0:
            senders_list = req.sender_names
        else:
            s_rows = cur.execute("""
                SELECT sender_name
                FROM messages
                WHERE sender_name IS NOT NULL AND sender_name != '' AND LOWER(sender_name) NOT LIKE 'wa%'
                GROUP BY sender_name
                ORDER BY count(*) DESC
            """).fetchall()
            senders_list = [r["sender_name"] for r in s_rows]

        for s_name in senders_list:
            row = cur.execute("""
                SELECT id, sender_name, raw_text
                FROM messages
                WHERE sender_name = ?
                ORDER BY id DESC
                LIMIT 1
            """, (s_name,)).fetchone()
            if row:
                target_msgs.append(row)
    elif req.mode == "all":
        limit = req.limit or 500
        if req.sender_names and len(req.sender_names) > 0:
            placeholders = ",".join("?" for _ in req.sender_names)
            query = f"""
                SELECT id, sender_name, raw_text
                FROM messages
                WHERE sender_name IN ({placeholders})
                ORDER BY id ASC
                LIMIT ?
            """
            target_msgs = cur.execute(query, req.sender_names + [limit]).fetchall()
        else:
            query = """
                SELECT id, sender_name, raw_text
                FROM messages
                WHERE sender_name IS NOT NULL AND LOWER(sender_name) NOT LIKE 'wa%'
                ORDER BY id ASC
                LIMIT ?
            """
            target_msgs = cur.execute(query, (limit,)).fetchall()

    conn.close()

    # Pre-group templates by sender_name for fast channel-first iteration
    templates_by_sender: Dict[str, List[Dict[str, Any]]] = {}
    global_templates: List[Dict[str, Any]] = []
    for tpl in all_templates:
        pat_str = (tpl.get("pattern_syntax") or "").strip()
        if not pat_str:
            continue
        s_key = (tpl.get("sender_name") or "").strip()
        if s_key:
            templates_by_sender.setdefault(s_key, []).append(tpl)
        else:
            global_templates.append(tpl)

    total_tested = len(target_msgs)
    matched_count = 0
    failed_count = 0
    signals_total_count = 0
    total_latency_us = 0
    summary_by_sender: Dict[str, SenderBatchStats] = {}
    results: List[BatchItemResult] = []

    for row in target_msgs:
        mid = row["id"]
        s_name = row["sender_name"] or "Desconocido"
        raw_text = row["raw_text"]

        if s_name not in summary_by_sender:
            summary_by_sender[s_name] = SenderBatchStats(sender_name=s_name)
        stats_obj = summary_by_sender[s_name]
        stats_obj.tested += 1

        # Build ordered candidate list for this message's channel:
        # 1. All templates registered for this channel (s_name)
        # 2. The active editor pattern_syntax (if provided and not already tested)
        # 3. Global templates and remaining templates
        candidates: List[Dict[str, Any]] = []
        seen_patterns = set()

        if req.iterate_channel_templates:
            for tpl in templates_by_sender.get(s_name, []):
                p = (tpl["pattern_syntax"] or "").strip()
                if p and p not in seen_patterns:
                    seen_patterns.add(p)
                    candidates.append(tpl)

        if pattern and pattern not in seen_patterns:
            seen_patterns.add(pattern)
            candidates.append({
                "id": None,
                "name": "Template Editor",
                "pattern_syntax": pattern,
                "sender_name": s_name,
            })

        if req.iterate_channel_templates:
            for tpl in global_templates:
                p = (tpl["pattern_syntax"] or "").strip()
                if p and p not in seen_patterns:
                    seen_patterns.add(p)
                    candidates.append(tpl)
            for tpl in all_templates:
                p = (tpl["pattern_syntax"] or "").strip()
                if p and p not in seen_patterns:
                    seen_patterns.add(p)
                    candidates.append(tpl)

        json_str = None
        sig_count = 0
        msg_latency_us = 0
        matched_tpl_id = None
        matched_tpl_name = None

        for cand in candidates:
            c_json, c_cnt, c_us = rust_bridge.parse_template_exact_sync(
                raw_text, cand["pattern_syntax"], req.end_char_new_line
            )
            msg_latency_us += c_us
            if c_json is not None and c_cnt > 0:
                json_str = c_json
                sig_count = c_cnt
                matched_tpl_id = cand["id"]
                matched_tpl_name = cand["name"]
                break

        total_latency_us += msg_latency_us
        is_match = (json_str is not None)

        if is_match:
            matched_count += 1
            signals_total_count += sig_count
            stats_obj.matched += 1
            stats_obj.signals_count += sig_count
        else:
            failed_count += 1
            stats_obj.failed += 1

        preview = raw_text.strip().replace("\n", " ")
        if len(preview) > 100:
            preview = preview[:100] + "..."

        results.append(BatchItemResult(
            message_id=mid,
            sender_name=s_name,
            raw_text_preview=preview,
            is_match=is_match,
            signals_count=sig_count,
            latency_us=msg_latency_us,
            signals_json=json_str,
            matched_template_id=matched_tpl_id,
            matched_template_name=matched_tpl_name,
        ))

    rate_pct = round((matched_count / total_tested * 100.0), 2) if total_tested > 0 else 0.0
    avg_us = round(total_latency_us / total_tested, 1) if total_tested > 0 else 0.0

    return TestBatchResponse(
        mode=req.mode,
        pattern_syntax=pattern or "Templates del canal",
        total_tested=total_tested,
        matched_count=matched_count,
        failed_count=failed_count,
        signals_total_count=signals_total_count,
        match_rate_pct=rate_pct,
        avg_latency_us=avg_us,
        summary_by_sender=summary_by_sender,
        results=results,
    )

@router.post("/validate")
async def save_validation(req: ValidateRequest):
    """
    Saves the user ground truth verification into validated_test_cases
    and marks the message as validated.
    """
    conn = get_db_connection()
    cur = conn.cursor()

    # Fetch message details
    msg_row = cur.execute("SELECT raw_text, sender_name FROM messages WHERE id = ?", (req.message_id,)).fetchone()
    if not msg_row:
        conn.close()
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Message not found")

    raw_text = msg_row[0]
    sender_name = msg_row[1]

    # Validate ground truth JSON syntax
    try:
        json.loads(req.ground_truth_json)
    except Exception as e:
        conn.close()
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail=f"Invalid JSON syntax in ground truth: {e}")

    # Insert or update in validated_test_cases
    cur.execute("""
        INSERT INTO validated_test_cases (
            message_id, sender_name, raw_text, is_signal, signals_count,
            ground_truth_json, parse_template_json, parse_signal_json,
            template_id, template_name, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        ON CONFLICT(message_id) DO UPDATE SET
            is_signal = excluded.is_signal,
            signals_count = excluded.signals_count,
            ground_truth_json = excluded.ground_truth_json,
            parse_template_json = excluded.parse_template_json,
            parse_signal_json = excluded.parse_signal_json,
            template_id = excluded.template_id,
            template_name = excluded.template_name,
            updated_at = CURRENT_TIMESTAMP
    """, (
        req.message_id,
        sender_name,
        raw_text,
        1 if req.is_signal else 0,
        req.signals_count,
        req.ground_truth_json,
        req.parse_template_json,
        req.parse_signal_json,
        req.template_id,
        req.template_name,
    ))

    # Update message status
    cur.execute("""
        UPDATE messages
        SET is_validated = 1,
            is_signal = ?
        WHERE id = ?
    """, (1 if req.is_signal else 0, req.message_id))

    conn.commit()
    conn.close()

    return {"status": "success", "message_id": req.message_id, "is_signal": req.is_signal}

@router.post("/batch-validate")
async def batch_validate_messages(req: BatchValidateRequest):
    """
    Bulk marks multiple selected messages (e.g. marking multiple messages as non-signal).
    """
    if not req.message_ids:
        return {"status": "success", "updated_count": 0}

    conn = get_db_connection()
    cur = conn.cursor()

    updated = 0
    for mid in req.message_ids:
        msg_row = cur.execute("SELECT raw_text, sender_name FROM messages WHERE id = ?", (mid,)).fetchone()
        if not msg_row:
            continue

        raw_text = msg_row[0]
        sender_name = msg_row[1]

        cur.execute("""
            INSERT INTO validated_test_cases (
                message_id, sender_name, raw_text, is_signal, signals_count,
                ground_truth_json, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(message_id) DO UPDATE SET
                is_signal = excluded.is_signal,
                signals_count = excluded.signals_count,
                ground_truth_json = excluded.ground_truth_json,
                updated_at = CURRENT_TIMESTAMP
        """, (
            mid,
            sender_name,
            raw_text,
            1 if req.is_signal else 0,
            req.signals_count,
            req.ground_truth_json,
        ))

        cur.execute("""
            UPDATE messages
            SET is_validated = 1,
                is_signal = ?
            WHERE id = ?
        """, (1 if req.is_signal else 0, mid))
        updated += 1

    conn.commit()
    conn.close()
    return {"status": "success", "updated_count": updated}

@router.get("/export-tests")
async def export_tests(
    only_signals: bool = Query(default=False),
    format_type: str = Query(default="suite", alias="format")
):
    """
    Exports verified ground-truth test cases into JSON format.
    Also saves a local copy at data/verified_signals_test.json.
    """
    conn = get_db_connection(row_factory=True)
    cur = conn.cursor()

    condition = "WHERE is_signal = 1" if only_signals else ""
    rows = cur.execute(f"""
        SELECT id, message_id, sender_name, raw_text, is_signal, signals_count,
               ground_truth_json, parse_template_json, parse_signal_json,
               template_name, created_at
        FROM validated_test_cases
        {condition}
        ORDER BY id ASC
    """).fetchall()

    conn.close()

    if format_type == "signals_json":
        # Group by sender for backward-compatible format with test_signals_json.py
        data_cases: Dict[str, List[Dict[str, Any]]] = {}
        for r in rows:
            s_name = r["sender_name"] or "General"
            clean_sender = "".join(c if c.isalnum() or c == "_" else "_" for c in s_name).strip("_") or "Sender"
            if clean_sender not in data_cases:
                data_cases[clean_sender] = []

            try:
                parsed_gt = json.loads(r["ground_truth_json"])
            except Exception:
                parsed_gt = []

            data_cases[clean_sender].append({
                "text": r["raw_text"],
                "has_signal": bool(r["is_signal"]),
                "signals_count": r["signals_count"],
                "signals": parsed_gt if isinstance(parsed_gt, list) else [parsed_gt],
                "template_name": r["template_name"],
            })

        export_data = {"dataCases": data_cases}
    else:
        # Comprehensive test suite format
        cases = []
        for r in rows:
            try:
                gt = json.loads(r["ground_truth_json"])
            except Exception:
                gt = r["ground_truth_json"]

            cases.append({
                "id": r["id"],
                "message_id": r["message_id"],
                "sender_name": r["sender_name"],
                "text": r["raw_text"],
                "is_signal": bool(r["is_signal"]),
                "signals_count": r["signals_count"],
                "ground_truth": gt,
                "template_name": r["template_name"],
                "created_at": r["created_at"],
            })

        export_data = {
            "metadata": {
                "generated_at": time.strftime("%Y-%m-%d %H:%M:%S UTC", time.gmtime()),
                "total_cases": len(cases),
                "signals_count": sum(1 for c in cases if c["is_signal"]),
                "non_signals_count": sum(1 for c in cases if not c["is_signal"]),
            },
            "cases": cases,
        }

    formatted_json = json.dumps(export_data, indent=2, ensure_ascii=False)

    # Save to local file
    output_path = settings.DATA_DIR / "verified_signals_test.json"
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(formatted_json)

    return Response(
        content=formatted_json,
        media_type="application/json",
        headers={
            "Content-Disposition": 'attachment; filename="verified_signals_test.json"'
        }
    )
