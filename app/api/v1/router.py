from fastapi import APIRouter
from app.api.v1.endpoints import auth, tokenizer, templates, benchmarks
from app.core.config import settings

api_router = APIRouter()

api_router.include_router(auth.router)
api_router.include_router(tokenizer.router)
api_router.include_router(templates.router)
api_router.include_router(benchmarks.router)

# Shorthand aliases to prevent 404s from reverse proxies or clients
api_router.add_api_route(
    "/tokenize",
    tokenizer.tokenize_endpoint,
    methods=["POST"],
    tags=["Tokens & Lexical Analysis"],
    include_in_schema=False,
)

@api_router.get("/health", tags=["Health"], include_in_schema=False)
async def api_v1_health():
    return {
        "status": "healthy",
        "service": settings.PROJECT_NAME,
        "version": settings.VERSION,
        "engine": "Rust (PyO3) + FastAPI (Async)",
    }
