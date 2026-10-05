import asyncio
import time
from typing import List, Dict, Any, Optional, Tuple
import rust_core

class RustBridgeService:
    """
    High-performance bridge to the native Rust engine (rust_core).
    All CPU-intensive calls are offloaded asynchronously via asyncio.to_thread
    so that FastAPI's event loop remains completely non-blocking.
    """

    @staticmethod
    def tokenize_sync(text: str) -> Tuple[List[Dict[str, Any]], int]:
        """Synchronously tokenize text and measure latency in microseconds."""
        t0 = time.perf_counter_ns()
        tokens_obj = rust_core.tokenize_text(text)
        duration_us = (time.perf_counter_ns() - t0) // 1000

        tokens = [
            {
                "kind": t.kind,
                "value": t.value,
                "start": t.start,
                "end": t.end,
                "line": t.line,
            }
            for t in tokens_obj
        ]
        return tokens, duration_us

    @classmethod
    async def tokenize_async(cls, text: str) -> Tuple[List[Dict[str, Any]], int]:
        return await asyncio.to_thread(cls.tokenize_sync, text)

    @staticmethod
    def extract_entities_sync(text: str) -> Tuple[Dict[str, Any], int]:
        """Synchronously extract entities and measure latency in microseconds."""
        t0 = time.perf_counter_ns()
        entities = rust_core.extract_entities(text)
        duration_us = (time.perf_counter_ns() - t0) // 1000
        return entities, duration_us

    @classmethod
    async def extract_entities_async(cls, text: str) -> Tuple[Dict[str, Any], int]:
        return await asyncio.to_thread(cls.extract_entities_sync, text)

    @staticmethod
    def match_template_sync(text: str, pattern: str, template_name: Optional[str] = None) -> Tuple[Optional[Dict[str, Any]], int]:
        """Synchronously evaluate text against a template pattern."""
        t0 = time.perf_counter_ns()
        res = rust_core.match_template(text, pattern, template_name)
        duration_us = (time.perf_counter_ns() - t0) // 1000

        if res is not None:
            return {
                "symbol": res.symbol,
                "action": res.action,
                "entry_prices": res.entry_prices,
                "stoploss": res.stoploss,
                "takeprofits": res.takeprofits,
                "timeframe": res.timeframe,
                "expiration": res.expiration,
                "is_binary": res.is_binary,
                "template_name": res.template_name,
                "matched_pattern": res.matched_pattern,
            }, duration_us
        return None, duration_us

    @classmethod
    async def match_template_async(cls, text: str, pattern: str, template_name: Optional[str] = None) -> Tuple[Optional[Dict[str, Any]], int]:
        return await asyncio.to_thread(cls.match_template_sync, text, pattern, template_name)

rust_bridge = RustBridgeService()
