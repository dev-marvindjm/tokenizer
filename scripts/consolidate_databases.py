#!/usr/bin/env python3
"""
Consolidation script:
Combines Telegram messages, templates, and senders from:
  1) /Users/macseqoia/gits/others/app/trading_signals.db
  2) /Users/macseqoia/gits/tokenizer/data/trading_signals_app.db
into a single clean SQLite database at:
  /Users/macseqoia/gits/tokenizer/data/consolidated_signals.db
"""

import sqlite3
import os
import sys
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
DATA_DIR = BASE_DIR / "data"

OTHERS_DB = Path("/Users/macseqoia/gits/others/app/trading_signals.db")
TOKENIZER_DB = DATA_DIR / "trading_signals_app.db"
TARGET_DB = DATA_DIR / "consolidated_signals.db"

def init_target_db(conn: sqlite3.Connection):
    cursor = conn.cursor()
    cursor.executescript("""
    PRAGMA foreign_keys = ON;

    CREATE TABLE IF NOT EXISTS messages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        source_db TEXT NOT NULL,
        original_id INTEGER,
        sender_id INTEGER,
        sender_name TEXT,
        channel_handle TEXT,
        external_message_id TEXT,
        raw_text TEXT NOT NULL,
        message_url TEXT,
        published_at TEXT,
        captured_at TEXT,
        processed_status TEXT DEFAULT 'new',
        is_validated BOOLEAN DEFAULT 0,
        is_signal BOOLEAN DEFAULT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_name);
    CREATE INDEX IF NOT EXISTS idx_messages_validated ON messages(is_validated);
    CREATE INDEX IF NOT EXISTS idx_messages_signal ON messages(is_signal);
    CREATE INDEX IF NOT EXISTS idx_messages_source_orig ON messages(source_db, original_id);

    CREATE TABLE IF NOT EXISTS templates (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        source_db TEXT NOT NULL,
        original_id INTEGER,
        name TEXT NOT NULL,
        pattern_syntax TEXT NOT NULL,
        template_type TEXT DEFAULT 'market',
        description TEXT,
        example_text TEXT,
        priority INTEGER DEFAULT 100,
        is_active BOOLEAN DEFAULT 1,
        sender_name TEXT
    );

    CREATE INDEX IF NOT EXISTS idx_templates_name ON templates(name);
    CREATE INDEX IF NOT EXISTS idx_templates_active ON templates(is_active);

    CREATE TABLE IF NOT EXISTS senders (
        id INTEGER PRIMARY KEY,
        display_name TEXT NOT NULL,
        handle TEXT,
        sender_type TEXT,
        messages_count INTEGER DEFAULT 0
    );

    CREATE TABLE IF NOT EXISTS validated_test_cases (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        message_id INTEGER UNIQUE,
        sender_name TEXT,
        raw_text TEXT NOT NULL,
        is_signal BOOLEAN NOT NULL,
        signals_count INTEGER NOT NULL DEFAULT 0,
        ground_truth_json TEXT NOT NULL,
        parse_template_json TEXT,
        parse_signal_json TEXT,
        template_id INTEGER,
        template_name TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
    );

    CREATE INDEX IF NOT EXISTS idx_test_cases_signal ON validated_test_cases(is_signal);
    CREATE INDEX IF NOT EXISTS idx_test_cases_msg_id ON validated_test_cases(message_id);

    -- Backward compatibility view
    CREATE VIEW IF NOT EXISTS source_messages AS SELECT * FROM messages;
    """)
    conn.commit()

def consolidate():
    DATA_DIR.mkdir(parents=True, exist_ok=True)

    print(f"📦 Initializing target database: {TARGET_DB}")
    # If target DB already exists, we back it up or recreate
    target_conn = sqlite3.connect(TARGET_DB)
    init_target_db(target_conn)
    target_cur = target_conn.cursor()

    # Track seen message texts to avoid duplicate insertions if desired
    seen_messages = set()
    total_messages_imported = 0
    total_templates_imported = 0
    total_senders_imported = 0

    # 1. Import Senders and Messages from others/app/trading_signals.db
    if OTHERS_DB.exists():
        print(f"📖 Importing from {OTHERS_DB}...")
        others_conn = sqlite3.connect(OTHERS_DB)
        others_conn.row_factory = sqlite3.Row
        others_cur = others_conn.cursor()

        # Senders map
        senders_map = {}
        try:
            for s in others_cur.execute("SELECT id, display_name, handle, sender_type FROM senders"):
                senders_map[s["id"]] = {
                    "name": s["display_name"],
                    "handle": s["handle"],
                    "type": s["sender_type"]
                }
                target_cur.execute(
                    """
                    INSERT OR REPLACE INTO senders (id, display_name, handle, sender_type)
                    VALUES (?, ?, ?, ?)
                    """,
                    (s["id"], s["display_name"], s["handle"], s["sender_type"])
                )
                total_senders_imported += 1
        except Exception as e:
            print(f"⚠️ Warning importing senders: {e}")

        # Messages
        try:
            msg_rows = others_cur.execute("""
                SELECT id, sender_id, external_message_id, raw_text, message_url, published_at, captured_at, processed_status
                FROM source_messages
                WHERE raw_text IS NOT NULL AND trim(raw_text) != ''
                ORDER BY id ASC
            """).fetchall()

            for row in msg_rows:
                s_info = senders_map.get(row["sender_id"], {})
                sender_name = s_info.get("name", f"Sender #{row['sender_id']}")
                handle = s_info.get("handle", "")

                target_cur.execute("""
                    INSERT INTO messages (
                        source_db, original_id, sender_id, sender_name, channel_handle,
                        external_message_id, raw_text, message_url, published_at, captured_at, processed_status
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """, (
                    "trading_signals.db",
                    row["id"],
                    row["sender_id"],
                    sender_name,
                    handle,
                    row["external_message_id"],
                    row["raw_text"],
                    row["message_url"],
                    row["published_at"],
                    row["captured_at"],
                    row["processed_status"]
                ))
                seen_messages.add(row["raw_text"].strip())
                total_messages_imported += 1

            print(f"  ✓ Imported {len(msg_rows)} messages from trading_signals.db")
        except Exception as e:
            print(f"⚠️ Error reading source_messages: {e}")

        # Templates from others
        try:
            tpl_rows = others_cur.execute("""
                SELECT id, sender_id, name, description, pattern, priority, is_active, example_text, template_type
                FROM signal_templates
                ORDER BY id ASC
            """).fetchall()

            seen_tpl_patterns = set()
            for tpl in tpl_rows:
                s_info = senders_map.get(tpl["sender_id"], {})
                s_name = s_info.get("name")
                pat = (tpl["pattern"] or "").strip()
                if not pat:
                    continue

                target_cur.execute("""
                    INSERT INTO templates (
                        source_db, original_id, name, pattern_syntax, template_type,
                        description, example_text, priority, is_active, sender_name
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """, (
                    "trading_signals.db",
                    tpl["id"],
                    tpl["name"],
                    pat,
                    tpl["template_type"] or "market",
                    tpl["description"],
                    tpl["example_text"],
                    tpl["priority"] or 100,
                    bool(tpl["is_active"]),
                    s_name
                ))
                seen_tpl_patterns.add((tpl["name"], pat))
                total_templates_imported += 1

            print(f"  ✓ Imported {len(tpl_rows)} templates from trading_signals.db")
        except Exception as e:
            print(f"⚠️ Error reading signal_templates from others: {e}")

        others_conn.close()
    else:
        print(f"⚠️ File not found: {OTHERS_DB}")

    # 2. Import Templates and records from tokenizer/data/trading_signals_app.db
    if TOKENIZER_DB.exists():
        print(f"📖 Importing from {TOKENIZER_DB}...")
        tok_conn = sqlite3.connect(TOKENIZER_DB)
        tok_conn.row_factory = sqlite3.Row
        tok_cur = tok_conn.cursor()

        try:
            tpl_rows = tok_cur.execute("""
                SELECT id, name, pattern_syntax, template_type, description, example_text,
                       priority, is_active, sender_name
                FROM signal_templates
                ORDER BY id ASC
            """).fetchall()

            tok_tpl_count = 0
            for tpl in tpl_rows:
                pat = (tpl["pattern_syntax"] or "").strip()
                name = tpl["name"]
                if not pat:
                    continue

                # Check if template already exists with same name and pattern
                exists = target_cur.execute(
                    "SELECT id FROM templates WHERE name = ? AND pattern_syntax = ?",
                    (name, pat)
                ).fetchone()

                if not exists:
                    target_cur.execute("""
                        INSERT INTO templates (
                            source_db, original_id, name, pattern_syntax, template_type,
                            description, example_text, priority, is_active, sender_name
                        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """, (
                        "trading_signals_app.db",
                        tpl["id"],
                        name,
                        pat,
                        tpl["template_type"] or "market",
                        tpl["description"],
                        tpl["example_text"],
                        tpl["priority"] or 100,
                        bool(tpl["is_active"]),
                        tpl["sender_name"]
                    ))
                    total_templates_imported += 1
                    tok_tpl_count += 1

            print(f"  ✓ Imported {tok_tpl_count} additional unique templates from trading_signals_app.db")
        except Exception as e:
            print(f"⚠️ Error reading signal_templates from tokenizer: {e}")

        # Check tokenized_records for any novel messages
        try:
            rec_rows = tok_cur.execute("""
                SELECT id, raw_text, template_id, created_at
                FROM tokenized_records
                WHERE raw_text IS NOT NULL AND trim(raw_text) != ''
            """).fetchall()

            rec_added = 0
            for rec in rec_rows:
                text_clean = rec["raw_text"].strip()
                if text_clean not in seen_messages:
                    target_cur.execute("""
                        INSERT INTO messages (
                            source_db, original_id, sender_id, sender_name, channel_handle,
                            external_message_id, raw_text, captured_at, processed_status
                        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """, (
                        "trading_signals_app.db",
                        rec["id"],
                        None,
                        "Tokenizer Sample Record",
                        "",
                        f"record_{rec['id']}",
                        rec["raw_text"],
                        rec["created_at"],
                        "new"
                    ))
                    seen_messages.add(text_clean)
                    total_messages_imported += 1
                    rec_added += 1

            if rec_added > 0:
                print(f"  ✓ Imported {rec_added} additional messages from tokenized_records")
        except Exception as e:
            print(f"⚠️ Error reading tokenized_records: {e}")

        tok_conn.close()

    # Update senders message count
    target_cur.execute("""
        UPDATE senders
        SET messages_count = (
            SELECT COUNT(*) FROM messages WHERE messages.sender_id = senders.id
        )
    """)

    target_conn.commit()

    # Final summary check
    total_msgs = target_cur.execute("SELECT count(*) FROM messages").fetchone()[0]
    total_tpls = target_cur.execute("SELECT count(*) FROM templates").fetchone()[0]
    total_snds = target_cur.execute("SELECT count(*) FROM senders").fetchone()[0]

    target_conn.close()

    print("\n" + "=" * 60)
    print("✅ CONSOLIDATION COMPLETE")
    print(f"🎯 Target Database: {TARGET_DB}")
    print(f"💬 Total Messages:  {total_msgs:,}")
    print(f"📋 Total Templates: {total_tpls:,}")
    print(f"📡 Total Senders:   {total_snds:,}")
    print("=" * 60)

if __name__ == "__main__":
    consolidate()
