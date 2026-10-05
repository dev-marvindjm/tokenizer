#!/bin/bash
set -e

echo "=== Starting Trading Tokenizer Container ==="

# Run database table initialization and template seeding if requested
if [ "${RUN_SEED:-true}" = "true" ]; then
    echo "[*] Initializing tables and seeding templates..."
    python -m app.scripts.seed_templates || echo "[!] Seed completed or templates already present."
fi

# Start FastAPI via Uvicorn
PORT_NUM="${PORT:-8000}"
echo "[*] Launching Uvicorn on 0.0.0.0:${PORT_NUM}..."
exec uvicorn app.main:app --host 0.0.0.0 --port "${PORT_NUM}" "$@"
