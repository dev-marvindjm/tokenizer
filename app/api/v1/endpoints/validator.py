import json
import sqlite3
import time
from pathlib import Path
from typing import List, Dict, Any, Optional
from fastapi import APIRouter, HTTPException, Query, Response, status
from pydantic import BaseModel, Field

from app.core.config import settings
from app.services.rust_bridge import rust_bridge

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

class ParseRequest(BaseModel):
    text: str
    template_id: Optional[int] = None
    pattern_syntax: Optional[str] = None
    template_name: Optional[str] = None

class ParseResponse(BaseModel):
    parse_template_json: Optional[str] = None
    parse_template_us: int = 0
    parse_signal_json: Optional[str] = None
    parse_signal_us: int = 0
    matched_template_id: Optional[int] = None
    matched_template_name: Optional[str] = None
    match_score: Optional[float] = None
    detected_signal_count: int = 0

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

    conn.close()
    return StatsResponse(
        total_messages=total_msgs,
        total_templates=total_tpls,
        total_senders=total_snds,
        total_validated=total_val,
        signals_count=sig_count,
        non_signals_count=non_sig_count,
    )

@router.get("/messages", response_model=MessageListResponse)
async def get_messages(
    page: int = Query(default=1, ge=1),
    page_size: int = Query(default=50, ge=1, le=200),
    sender_name: Optional[str] = None,
    status_filter: str = Query(default="all", alias="status"),
    search: Optional[str] = None,
):
    """
    Paginated list of messages from consolidated_signals.db with filtering.
    status_filter options: 'all', 'pending', 'validated', 'signal', 'non_signal'
    """
    conn = get_db_connection(row_factory=True)
    cur = conn.cursor()

    conditions = []
    params = []

    if sender_name:
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
               m.captured_at, m.processed_status, m.is_validated, m.is_signal,
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
               m.captured_at, m.processed_status, m.is_validated, m.is_signal,
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
async def get_templates():
    """Returns all available templates for pattern testing."""
    conn = get_db_connection(row_factory=True)
    cur = conn.cursor()

    rows = cur.execute("""
        SELECT id, name, pattern_syntax, template_type, description,
               example_text, priority, is_active, sender_name
        FROM templates
        ORDER BY priority DESC, id ASC
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

@router.get("/senders", response_model=List[SenderItem])
async def get_senders():
    """Returns list of distinct senders/channels ordered by message count."""
    conn = get_db_connection(row_factory=True)
    cur = conn.cursor()

    rows = cur.execute("""
        SELECT sender_name, channel_handle, count(*) as count
        FROM messages
        WHERE sender_name IS NOT NULL AND sender_name != ''
        GROUP BY sender_name
        ORDER BY count DESC
    """).fetchall()

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
    If no template is specified, automatically searches active templates for best match.
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
            signals_arr = parsed_obj.get("signals", [])
            det_count = len(signals_arr)
        except Exception:
            det_count = 0

    # 2. Execute parse_template_json
    tpl_json = None
    tpl_us = 0
    matched_id = req.template_id
    matched_name = req.template_name
    match_score = None

    conn = get_db_connection(row_factory=True)
    cur = conn.cursor()

    if req.pattern_syntax:
        # Pattern provided directly
        tpl_json, tpl_us = await rust_bridge.parse_template_async(
            text, req.pattern_syntax, req.template_name
        )
    elif req.template_id:
        # Template specified by ID
        row = cur.execute("SELECT name, pattern_syntax FROM templates WHERE id = ?", (req.template_id,)).fetchone()
        if row:
            matched_name = row["name"]
            tpl_json, tpl_us = await rust_bridge.parse_template_async(
                text, row["pattern_syntax"], matched_name
            )
    else:
        # Auto-match: test against active templates to find best score
        active_tpls = cur.execute(
            "SELECT id, name, pattern_syntax FROM templates WHERE is_active = 1 ORDER BY priority DESC LIMIT 50"
        ).fetchall()

        best_score = -1.0
        best_json = None
        best_id = None
        best_name = None
        best_time = 0

        for tpl in active_tpls:
            res_str, d_us = await rust_bridge.parse_template_async(
                text, tpl["pattern_syntax"], tpl["name"]
            )
            if res_str:
                try:
                    obj = json.loads(res_str)
                    sc = float(obj.get("match_score", 0.0))
                    if sc > best_score:
                        best_score = sc
                        best_json = res_str
                        best_id = tpl["id"]
                        best_name = tpl["name"]
                        best_time = d_us
                        if best_score >= 100.0:
                            break
                except Exception:
                    pass

        if best_json is not None:
            tpl_json = best_json
            tpl_us = best_time
            matched_id = best_id
            matched_name = best_name
            match_score = best_score

    conn.close()

    # Extract score if tpl_json present and match_score not already extracted
    if tpl_json and match_score is None:
        try:
            obj = json.loads(tpl_json)
            match_score = float(obj.get("match_score", 0.0))
        except Exception:
            pass

    return ParseResponse(
        parse_template_json=tpl_json,
        parse_template_us=tpl_us,
        parse_signal_json=sig_json,
        parse_signal_us=sig_us,
        matched_template_id=matched_id,
        matched_template_name=matched_name,
        match_score=match_score,
        detected_signal_count=det_count,
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
