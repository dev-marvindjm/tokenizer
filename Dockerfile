# ==========================================
# Stage 1: Build Native Rust Core Wheel
# ==========================================
FROM rust:1.85-slim AS builder

WORKDIR /build

ENV PYO3_NO_PYTHON=1

# Copy configuration and source files required for compiling rust_core
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY data ./data

# Compile optimized release cdylib library
RUN cargo build --release

# ==========================================
# Stage 2: Final Runtime Application
# ==========================================
FROM python:3.12-slim AS runner

WORKDIR /app

RUN mkdir -p /app/data

ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PORT=8000 \
    DATABASE_URL="sqlite+aiosqlite:////app/data/trading_signals_app.db"

# Install curl for healthchecks
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Python requirements
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

# Install the native rust_core library built in builder stage
COPY --from=builder /build/target/release/librust_core.so /usr/local/lib/python3.12/site-packages/rust_core.abi3.so

# Copy application code, data, tests, and scripts
COPY app ./app
COPY data ./data
COPY tests ./tests
COPY entrypoint.sh ./entrypoint.sh
RUN chmod +x ./entrypoint.sh

EXPOSE 8000

HEALTHCHECK --interval=10s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

ENTRYPOINT ["./entrypoint.sh"]
