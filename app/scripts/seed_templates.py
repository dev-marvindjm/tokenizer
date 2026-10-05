import asyncio
from pathlib import Path
from sqlmodel import select, delete
from sqlalchemy import text
from app.core.config import settings
from app.core.database import init_db, async_session_factory
from app.core.security import get_password_hash
from app.domain.models import SignalTemplate, User

SLOT_TEMPLATES = [
    # A: Forex MT5
    { "id": 1, "code_slot": "A1", "section": "A-Forex MT5", "name": "EURUSD M1 Scalper", "sender_name": "VIP Scalpers FX", "template_type": "market", "pattern_syntax": "EURUSD $(action) $(entry)", "priority": 120, "is_active": True, "win_rate": 84.5, "latency_us": 28, "signals_count": 540, "quality_score": 92 },
    { "id": 2, "code_slot": "A2", "section": "A-Forex MT5", "name": "GBPUSD Breakout", "sender_name": "London Session", "template_type": "market", "pattern_syntax": "GBPUSD $(action) ENTRY $(entry) SL $(sl)", "priority": 110, "is_active": True, "win_rate": 79.2, "latency_us": 32, "signals_count": 420, "quality_score": 88 },
    { "id": 3, "code_slot": "A3", "section": "A-Forex MT5", "name": "USDJPY Trend Limit", "sender_name": "Tokyo Open", "template_type": "market", "pattern_syntax": "USDJPY LIMIT $(action) $(entry)", "priority": 95, "is_active": False, "win_rate": 68.0, "latency_us": 35, "signals_count": 190, "quality_score": 72 },
    { "id": 4, "code_slot": "A4", "section": "A-Forex MT5", "name": "AUDUSD Range MeanRev", "sender_name": "Aussie Quant", "template_type": "market", "pattern_syntax": "AUDUSD REVERSE $(action)", "priority": 90, "is_active": False, "win_rate": 65.4, "latency_us": 41, "signals_count": 140, "quality_score": 69 },
    { "id": 5, "code_slot": "A5", "section": "A-Forex MT5", "name": "NZDUSD Momentum", "sender_name": "Kiwi FX", "template_type": "market", "pattern_syntax": "NZDUSD $(action) NOW", "priority": 105, "is_active": True, "win_rate": 76.1, "latency_us": 30, "signals_count": 310, "quality_score": 82 },
    { "id": 6, "code_slot": "A6", "section": "A-Forex MT5", "name": "EURGBP Support Bounce", "sender_name": "Euro Swing", "template_type": "market", "pattern_syntax": "EURGBP BOUNCE $(action)", "priority": 100, "is_active": True, "win_rate": 81.0, "latency_us": 34, "signals_count": 280, "quality_score": 86 },
    { "id": 7, "code_slot": "A7", "section": "A-Forex MT5", "name": "USDCAD Oil Correl", "sender_name": "Commodity FX", "template_type": "market", "pattern_syntax": "USDCAD $(action) OIL_SPIKE", "priority": 85, "is_active": False, "win_rate": 62.0, "latency_us": 44, "signals_count": 95, "quality_score": 64 },
    { "id": 8, "code_slot": "A8", "section": "A-Forex MT5", "name": "EURJPY Volatility Break", "sender_name": "Yen Crosses", "template_type": "market", "pattern_syntax": "EURJPY BREAKOUT $(action)", "priority": 115, "is_active": True, "win_rate": 82.0, "latency_us": 29, "signals_count": 360, "quality_score": 89 },
    { "id": 9, "code_slot": "A9", "section": "A-Forex MT5", "name": "GBPJPY Dragon Ride", "sender_name": "Geppy VIP", "template_type": "market", "pattern_syntax": "GBPJPY $(action) $(entry) TP $(tp)", "priority": 80, "is_active": False, "win_rate": 64.2, "latency_us": 48, "signals_count": 110, "quality_score": 66 },
    { "id": 10, "code_slot": "A10", "section": "A-Forex MT5", "name": "USDCHF Safe Haven", "sender_name": "Swiss Vault", "template_type": "market", "pattern_syntax": "USDCHF $(action) NOW", "priority": 75, "is_active": False, "win_rate": 59.8, "latency_us": 52, "signals_count": 80, "quality_score": 60 },
    { "id": 11, "code_slot": "A11", "section": "A-Forex MT5", "name": "CADJPY Asian Scalp", "sender_name": "Tokyo Scalp", "template_type": "market", "pattern_syntax": "CADJPY $(action) M5", "priority": 70, "is_active": False, "win_rate": 61.5, "latency_us": 46, "signals_count": 75, "quality_score": 62 },
    { "id": 12, "code_slot": "A12", "section": "A-Forex MT5", "name": "AUDJPY High Beta", "sender_name": "Risk On Algo", "template_type": "market", "pattern_syntax": "AUDJPY $(action) SL $(sl)", "priority": 108, "is_active": True, "win_rate": 77.8, "latency_us": 33, "signals_count": 240, "quality_score": 84 },

    # B: Binary Options
    { "id": 13, "code_slot": "B1", "section": "B-Binary Turbo", "name": "EURUSD OTC 1M Call/Put", "sender_name": "Quotex VIP Club", "template_type": "binary", "pattern_syntax": "EURUSD_otc $(action) M1 GALE $(gale)", "priority": 130, "is_active": True, "win_rate": 86.4, "latency_us": 22, "signals_count": 980, "quality_score": 95 },
    { "id": 14, "code_slot": "B2", "section": "B-Binary Turbo", "name": "GBPUSD OTC 5M Expiry", "sender_name": "Pocket Option Elite", "template_type": "binary", "pattern_syntax": "GBPUSD_otc M5 $(action) $(time)", "priority": 85, "is_active": False, "win_rate": 67.0, "latency_us": 38, "signals_count": 210, "quality_score": 70 },
    { "id": 15, "code_slot": "B3", "section": "B-Binary Turbo", "name": "USDJPY OTC Gale Step 1", "sender_name": "Turbo Binary Pro", "template_type": "binary", "pattern_syntax": "USDJPY_otc $(action) M1", "priority": 125, "is_active": True, "win_rate": 83.2, "latency_us": 24, "signals_count": 670, "quality_score": 90 },
    { "id": 16, "code_slot": "B4", "section": "B-Binary Turbo", "name": "AUDCAD OTC Reversal", "sender_name": "Binary Master", "template_type": "binary", "pattern_syntax": "AUDCAD_otc REVERSE $(action)", "priority": 90, "is_active": False, "win_rate": 69.5, "latency_us": 36, "signals_count": 180, "quality_score": 73 },
    { "id": 17, "code_slot": "B5", "section": "B-Binary Turbo", "name": "NZDUSD OTC 30s Quick", "sender_name": "Turbo Flash", "template_type": "binary", "pattern_syntax": "NZDUSD_otc S30 $(action)", "priority": 80, "is_active": False, "win_rate": 63.8, "latency_us": 45, "signals_count": 120, "quality_score": 65 },
    { "id": 18, "code_slot": "B6", "section": "B-Binary Turbo", "name": "EURGBP OTC Safe Stride", "sender_name": "Quotex Safe", "template_type": "binary", "pattern_syntax": "EURGBP_otc $(action) M2", "priority": 115, "is_active": True, "win_rate": 81.5, "latency_us": 26, "signals_count": 510, "quality_score": 87 },
    { "id": 19, "code_slot": "B7", "section": "B-Binary Turbo", "name": "AUDCHF OTC Support", "sender_name": "Binary Snipers", "template_type": "binary", "pattern_syntax": "AUDCHF_otc LEVEL $(action)", "priority": 110, "is_active": True, "win_rate": 80.0, "latency_us": 29, "signals_count": 430, "quality_score": 86 },
    { "id": 20, "code_slot": "B8", "section": "B-Binary Turbo", "name": "GBPJPY OTC Momentum", "sender_name": "Geppy Turbo", "template_type": "binary", "pattern_syntax": "GBPJPY_otc $(action) GALE 2", "priority": 105, "is_active": True, "win_rate": 78.4, "latency_us": 31, "signals_count": 390, "quality_score": 83 },
    { "id": 21, "code_slot": "B9", "section": "B-Binary Turbo", "name": "USDCHF OTC Trend", "sender_name": "Safe Scalp", "template_type": "binary", "pattern_syntax": "USDCHF_otc $(action)", "priority": 100, "is_active": True, "win_rate": 77.0, "latency_us": 33, "signals_count": 310, "quality_score": 81 },
    { "id": 22, "code_slot": "B10", "section": "B-Binary Turbo", "name": "EURAUD OTC Asian", "sender_name": "Night Traders", "template_type": "binary", "pattern_syntax": "EURAUD_otc NIGHT $(action)", "priority": 80, "is_active": False, "win_rate": 64.0, "latency_us": 42, "signals_count": 140, "quality_score": 67 },
    { "id": 23, "code_slot": "B11", "section": "B-Binary Turbo", "name": "GBPAUD OTC Volatility", "sender_name": "High Range", "template_type": "binary", "pattern_syntax": "GBPAUD_otc $(action) M5", "priority": 75, "is_active": False, "win_rate": 62.5, "latency_us": 49, "signals_count": 90, "quality_score": 63 },
    { "id": 24, "code_slot": "B12", "section": "B-Binary Turbo", "name": "USDMXN OTC Speculative", "sender_name": "Exotic OTC", "template_type": "binary", "pattern_syntax": "USDMXN_otc $(action) M1", "priority": 118, "is_active": True, "win_rate": 82.6, "latency_us": 27, "signals_count": 480, "quality_score": 88 },

    # C: Crypto Futures
    { "id": 25, "code_slot": "C1", "section": "C-Crypto Scalp", "name": "BTCUSDT Perpetual Limit", "sender_name": "Crypto Futures VIP", "template_type": "market", "pattern_syntax": "BTCUSDT $(action) $(entry) SL $(sl)", "priority": 140, "is_active": True, "win_rate": 87.2, "latency_us": 19, "signals_count": 1250, "quality_score": 96 },
    { "id": 26, "code_slot": "C2", "section": "C-Crypto Scalp", "name": "ETHUSDT Breakout", "sender_name": "Ethereum Alpha", "template_type": "market", "pattern_syntax": "ETHUSDT BREAK $(action) $(entry)", "priority": 130, "is_active": True, "win_rate": 84.1, "latency_us": 21, "signals_count": 940, "quality_score": 93 },
    { "id": 27, "code_slot": "C3", "section": "C-Crypto Scalp", "name": "SOLUSDT High Volatility", "sender_name": "Solana Degens", "template_type": "market", "pattern_syntax": "SOLUSDT $(action) NOW", "priority": 120, "is_active": True, "win_rate": 81.3, "latency_us": 25, "signals_count": 620, "quality_score": 89 },
    { "id": 28, "code_slot": "C4", "section": "C-Crypto Scalp", "name": "BNBUSDT Binance Ecosystem", "sender_name": "BNB Club", "template_type": "market", "pattern_syntax": "BNBUSDT $(action) $(entry)", "priority": 115, "is_active": True, "win_rate": 79.5, "latency_us": 28, "signals_count": 410, "quality_score": 85 },
    { "id": 29, "code_slot": "C5", "section": "C-Crypto Scalp", "name": "XRPUSDT Ripple Swing", "sender_name": "Ripple Wave", "template_type": "market", "pattern_syntax": "XRPUSDT SWING $(action)", "priority": 110, "is_active": True, "win_rate": 78.0, "latency_us": 30, "signals_count": 360, "quality_score": 84 },
    { "id": 30, "code_slot": "C6", "section": "C-Crypto Scalp", "name": "DOGEUSDT Meme Breakout", "sender_name": "Meme Coins Elite", "template_type": "market", "pattern_syntax": "DOGEUSDT MOON $(action)", "priority": 105, "is_active": True, "win_rate": 76.4, "latency_us": 32, "signals_count": 330, "quality_score": 81 },
    { "id": 31, "code_slot": "C7", "section": "C-Crypto Scalp", "name": "ADAUSDT Trend Line", "sender_name": "Cardano Scalp", "template_type": "market", "pattern_syntax": "ADAUSDT $(action) $(entry)", "priority": 100, "is_active": True, "win_rate": 75.0, "latency_us": 34, "signals_count": 290, "quality_score": 80 },
    { "id": 32, "code_slot": "C8", "section": "C-Crypto Scalp", "name": "AVAXUSDT Avalanche Pump", "sender_name": "Avalanche Hub", "template_type": "market", "pattern_syntax": "AVAXUSDT $(action) NOW", "priority": 102, "is_active": True, "win_rate": 77.1, "latency_us": 33, "signals_count": 270, "quality_score": 82 },
    { "id": 33, "code_slot": "C9", "section": "C-Crypto Scalp", "name": "DOTUSDT Polkadot Parachain", "sender_name": "Polka Traders", "template_type": "market", "pattern_syntax": "DOTUSDT $(action)", "priority": 85, "is_active": False, "win_rate": 66.0, "latency_us": 42, "signals_count": 130, "quality_score": 68 },
    { "id": 34, "code_slot": "C10", "section": "C-Crypto Scalp", "name": "LINKUSDT Oracle Pulse", "sender_name": "Chainlink Core", "template_type": "market", "pattern_syntax": "LINKUSDT $(action) $(entry)", "priority": 80, "is_active": False, "win_rate": 64.5, "latency_us": 45, "signals_count": 110, "quality_score": 66 },
    { "id": 35, "code_slot": "C11", "section": "C-Crypto Scalp", "name": "NEARUSDT AI Protocol", "sender_name": "Near AI Traders", "template_type": "market", "pattern_syntax": "NEARUSDT $(action)", "priority": 75, "is_active": False, "win_rate": 61.0, "latency_us": 49, "signals_count": 85, "quality_score": 62 },
    { "id": 36, "code_slot": "C12", "section": "C-Crypto Scalp", "name": "SUIUSDT Layer 1 Speed", "sender_name": "Sui Network VIP", "template_type": "market", "pattern_syntax": "SUIUSDT SPEED $(action)", "priority": 70, "is_active": False, "win_rate": 60.5, "latency_us": 51, "signals_count": 70, "quality_score": 61 },

    # D: Commodities & Indices
    { "id": 37, "code_slot": "D1", "section": "D-Commodities", "name": "XAUUSD Gold London Scalp", "sender_name": "Gold Swing Pro", "template_type": "market", "pattern_syntax": "GOLD $(action) $(entry) SL $(sl) TP $(tp)", "priority": 150, "is_active": True, "win_rate": 88.9, "latency_us": 18, "signals_count": 1450, "quality_score": 97 },
    { "id": 38, "code_slot": "D2", "section": "D-Commodities", "name": "US30 Dow Jones NY Open", "sender_name": "Wall Street Index", "template_type": "market", "pattern_syntax": "US30 OPEN $(action) $(entry)", "priority": 85, "is_active": False, "win_rate": 67.5, "latency_us": 39, "signals_count": 220, "quality_score": 71 },
    { "id": 39, "code_slot": "D3", "section": "D-Commodities", "name": "NAS100 Nasdaq Tech Run", "sender_name": "Nasdaq Titans", "template_type": "market", "pattern_syntax": "NAS100 $(action) $(entry)", "priority": 80, "is_active": False, "win_rate": 66.0, "latency_us": 42, "signals_count": 190, "quality_score": 69 },
    { "id": 40, "code_slot": "D4", "section": "D-Commodities", "name": "GER40 DAX Morning Break", "sender_name": "Frankfurt Pulse", "template_type": "market", "pattern_syntax": "GER40 $(action) BREAKOUT", "priority": 120, "is_active": True, "win_rate": 82.4, "latency_us": 27, "signals_count": 510, "quality_score": 89 },
    { "id": 41, "code_slot": "D5", "section": "D-Commodities", "name": "USOIL WTI Crude Inventory", "sender_name": "Oil Masters", "template_type": "market", "pattern_syntax": "USOIL EIA $(action)", "priority": 115, "is_active": True, "win_rate": 80.5, "latency_us": 30, "signals_count": 420, "quality_score": 86 },
    { "id": 42, "code_slot": "D6", "section": "D-Commodities", "name": "XAGUSD Silver Volatility", "sender_name": "Silver Bullets", "template_type": "market", "pattern_syntax": "SILVER $(action) $(entry)", "priority": 110, "is_active": True, "win_rate": 78.8, "latency_us": 33, "signals_count": 360, "quality_score": 84 },
    { "id": 43, "code_slot": "D7", "section": "D-Commodities", "name": "UK100 FTSE London Close", "sender_name": "London Indices", "template_type": "market", "pattern_syntax": "UK100 CLOSE $(action)", "priority": 105, "is_active": True, "win_rate": 76.5, "latency_us": 36, "signals_count": 280, "quality_score": 82 },
    { "id": 44, "code_slot": "D8", "section": "D-Commodities", "name": "SPX500 S&P Momentum", "sender_name": "Macro Quant", "template_type": "market", "pattern_syntax": "SPX500 $(action) $(entry)", "priority": 100, "is_active": True, "win_rate": 77.2, "latency_us": 35, "signals_count": 310, "quality_score": 83 },
    { "id": 45, "code_slot": "D9", "section": "D-Commodities", "name": "JP225 Nikkei Asian Run", "sender_name": "Tokyo Exchange", "template_type": "market", "pattern_syntax": "JP225 $(action) OPEN", "priority": 75, "is_active": False, "win_rate": 63.0, "latency_us": 46, "signals_count": 110, "quality_score": 65 },
    { "id": 46, "code_slot": "D10", "section": "D-Commodities", "name": "NATGAS Natural Gas Surge", "sender_name": "Energy Desk", "template_type": "market", "pattern_syntax": "NATGAS $(action)", "priority": 70, "is_active": False, "win_rate": 61.2, "latency_us": 50, "signals_count": 85, "quality_score": 63 },
    { "id": 47, "code_slot": "D11", "section": "D-Commodities", "name": "COPPER Metal Breakout", "sender_name": "Metals Trader", "template_type": "market", "pattern_syntax": "COPPER $(action) $(entry)", "priority": 65, "is_active": False, "win_rate": 59.4, "latency_us": 54, "signals_count": 60, "quality_score": 60 },
    { "id": 48, "code_slot": "D12", "section": "D-Commodities", "name": "BRENT North Sea Oil", "sender_name": "Brent Crude Hub", "template_type": "market", "pattern_syntax": "BRENT $(action)", "priority": 108, "is_active": True, "win_rate": 79.0, "latency_us": 31, "signals_count": 340, "quality_score": 85 },
]

async def seed_templates():
    print("[*] Initializing database tables...")
    await init_db()

    async with async_session_factory() as session:
        # 1. Ensure initial admin / system user exists
        admin_stmt = select(User).where(User.username == settings.FIRST_SUPERUSER_USERNAME)
        admin_res = await session.exec(admin_stmt)
        admin_user = admin_res.first()

        if not admin_user:
            admin_user = User(
                email=settings.FIRST_SUPERUSER_EMAIL,
                username=settings.FIRST_SUPERUSER_USERNAME,
                hashed_password=get_password_hash(settings.FIRST_SUPERUSER_PASSWORD),
                full_name="System Administrator",
                is_active=True,
                is_superuser=True,
            )
            session.add(admin_user)
            await session.commit()
            await session.refresh(admin_user)
            print(f"[✓] Created initial superuser: {admin_user.username} ({admin_user.email})")

        # 2. Clean temporary / test data
        print("[*] Purging temporary / test / unassigned templates...")
        all_res = await session.exec(select(SignalTemplate))
        existing_all = all_res.all()
        slot_codes = {t["code_slot"] for t in SLOT_TEMPLATES}

        for ext in existing_all:
            # Purge test templates only
            if (
                ext.name.startswith("Integration Test")
                or ext.name.startswith("Beta Private")
            ):
                await session.delete(ext)

        await session.commit()

        # 3. Seed / sync the 48 assigned slot templates
        inserted_count = 0
        updated_count = 0

        for tpl_data in SLOT_TEMPLATES:
            stmt = select(SignalTemplate).where(
                (SignalTemplate.code_slot == tpl_data["code_slot"]) | (SignalTemplate.name == tpl_data["name"])
            )
            res = await session.exec(stmt)
            existing = res.first()

            if existing is None:
                new_tpl = SignalTemplate(
                    id=tpl_data["id"],
                    user_id=admin_user.id,
                    is_system=True,
                    name=tpl_data["name"],
                    code_slot=tpl_data["code_slot"],
                    section=tpl_data["section"],
                    sender_name=tpl_data["sender_name"],
                    template_type=tpl_data["template_type"],
                    pattern_syntax=tpl_data["pattern_syntax"],
                    priority=tpl_data["priority"],
                    is_active=tpl_data["is_active"],
                    win_rate=tpl_data["win_rate"],
                    latency_us=tpl_data["latency_us"],
                    signals_count=tpl_data["signals_count"],
                    quality_score=tpl_data["quality_score"],
                )
                session.add(new_tpl)
                inserted_count += 1
            else:
                existing.code_slot = tpl_data["code_slot"]
                existing.section = tpl_data["section"]
                existing.sender_name = tpl_data["sender_name"]
                existing.name = tpl_data["name"]
                existing.pattern_syntax = tpl_data["pattern_syntax"]
                existing.template_type = tpl_data["template_type"]
                existing.priority = tpl_data["priority"]
                existing.win_rate = tpl_data["win_rate"]
                existing.latency_us = tpl_data["latency_us"]
                existing.signals_count = tpl_data["signals_count"]
                existing.quality_score = tpl_data["quality_score"]
                existing.is_system = True
                existing.user_id = admin_user.id
                session.add(existing)
                updated_count += 1

        await session.commit()
        print(f"[✓] Seeding completed: {inserted_count} inserted, {updated_count} updated.")

if __name__ == "__main__":
    asyncio.run(seed_templates())
