#!/usr/bin/env python3
"""Quick TAPO P110 connectivity check.

Usage:
  TAPO_EMAIL=... TAPO_PASSWORD=... TAPO_DEVICE_IP=... python backend/scripts/test_tapo_connection.py

If you prefer a .env file, copy .env.example to .env and fill in the TAPO_* values.
"""
import asyncio
import logging
import os
import sys
from pathlib import Path

# Allow imports from backend/src and backend/config
ROOT = Path(__file__).resolve().parents[1]
sys.path.append(str(ROOT / "src"))
sys.path.append(str(ROOT))

from tapo_monitor import TapoMonitor  # type: ignore
from config.config import Config  # type: ignore

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)s %(message)s",
)
logger = logging.getLogger("tapo_test")


async def main():
    # Ensure env is loaded for config validation
    try:
        Config.validate()
    except ValueError as e:
        logger.error("Config validation failed: %s", e)
        return 1

    # Prefer explicit TAPO envs; fall back to Config
    email = os.getenv("TAPO_EMAIL", Config.TAPO_EMAIL)
    password = os.getenv("TAPO_PASSWORD", Config.TAPO_PASSWORD)
    device_ip = os.getenv("TAPO_DEVICE_IP", Config.TAPO_DEVICE_IP)

    if not (email and password and device_ip):
        logger.error("Missing TAPO_EMAIL / TAPO_PASSWORD / TAPO_DEVICE_IP. Aborting.")
        return 1

    monitor = TapoMonitor(email, password, device_ip)

    logger.info("Connecting to TAPO device at %s...", device_ip)
    if not await monitor.connect():
        logger.error("Failed to connect. Check credentials, IP reachability, and that the device is on the same LAN.")
        return 1

    logger.info("Connected. Fetching device info...")
    info = await monitor.get_device_info()
    logger.info("Device info: %s", info)

    logger.info("Fetching current power...")
    power = await monitor.get_current_power()
    logger.info("Current power: %s", power)

    logger.info("Fetching energy usage...")
    energy = await monitor.get_energy_usage()
    logger.info("Energy usage: %s", energy)

    logger.info("Success: TAPO API calls completed without exceptions.")
    return 0


if __name__ == "__main__":
    exit_code = asyncio.run(main())
    sys.exit(exit_code)
