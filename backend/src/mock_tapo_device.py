import asyncio
import logging
import random
import time
from datetime import datetime

logger = logging.getLogger(__name__)


class MockTapoDevice:
    """Lightweight mock for the Tapo P110 smart plug.

    Produces deterministic-but-jittered readings to exercise the oracle flow
    without real hardware. All methods mirror the async interface of
    :class:`tapo_monitor.TapoMonitor`.
    """

    def __init__(self, profile: str = "steady"):
        self.profile = profile
        self._start = time.time()
        self._energy_wh = 0.0
        self._device_on = True

    async def connect(self) -> bool:
        logger.info("🔌 Using mock TAPO device (profile=%s)", self.profile)
        await asyncio.sleep(0)  # allow scheduling
        return True

    def _current_power_watts(self) -> float:
        """Generate a pseudo-realistic current power value."""
        base = {
            "steady": 62.5,
            "bursty": 45.0,
            "idle": 18.0,
        }.get(self.profile, 62.5)
        jitter = random.uniform(-5.0, 12.0)
        if self.profile == "bursty":
            jitter += random.uniform(0, 55)
        return max(0.0, base + jitter)

    async def get_current_power(self) -> dict:
        power_w = self._current_power_watts()
        power_mw = int(power_w * 1000)
        return {
            "power_watts": power_w,
            "power_milliwatts": power_mw,
            "timestamp": datetime.now().isoformat(),
        }

    async def get_energy_usage(self) -> dict:
        """Approximate cumulative energy based on elapsed time and power."""
        elapsed_hours = (time.time() - self._start) / 3600.0
        # Assume average power around the current reading
        avg_power_w = max(10.0, self._current_power_watts() * 0.85)
        self._energy_wh = avg_power_w * elapsed_hours

        return {
            "today_energy_wh": round(self._energy_wh, 2),
            "today_energy_kwh": round(self._energy_wh / 1000.0, 4),
            "today_runtime_min": int(elapsed_hours * 60),
            "month_energy_wh": round(self._energy_wh * 4, 2),
            "month_energy_kwh": round(self._energy_wh * 4 / 1000.0, 4),
            "month_runtime_min": int(elapsed_hours * 60 * 4),
            "local_time": datetime.now().isoformat(),
        }

    async def get_device_info(self) -> dict:
        return {
            "device_id": "mock-device-001",
            "model": "P110-MOCK",
            "hardware_version": "1.0",
            "firmware_version": "1.0-mock",
            "mac_address": "00:11:22:33:44:55",
            "device_on": self._device_on,
            "rssi": -45,
        }

    async def get_complete_snapshot(self) -> dict:
        device_info = await self.get_device_info()
        current_power = await self.get_current_power()
        energy_usage = await self.get_energy_usage()
        return {
            "device_info": device_info,
            "current_power": current_power,
            "energy_usage": energy_usage,
            "snapshot_time": datetime.now().isoformat(),
        }

    async def turn_on(self):
        self._device_on = True
        return True

    async def turn_off(self):
        self._device_on = False
        return True
