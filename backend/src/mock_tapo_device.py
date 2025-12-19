"""Mock TAPO device for testing blockchain functionality without hardware"""

import asyncio
import random
from datetime import datetime
import logging

logger = logging.getLogger(__name__)


class MockTapoDevice:
    """Mock TAPO P110 device that simulates energy readings"""
    
    def __init__(self, device_id: str = "MOCK-DEVICE-001"):
        self.device_id = device_id
        self.connected = False
        self.power_watts = 0.0
        self.today_energy_wh = 0
        self.is_on = True
        
    async def connect(self) -> bool:
        """Simulate connection"""
        logger.info(f"🔌 Mock device connecting: {self.device_id}")
        await asyncio.sleep(0.1)  # Simulate network delay
        self.connected = True
        logger.info(f"✅ Mock device connected: {self.device_id}")
        return True
    
    async def get_current_power(self) -> dict:
        """Simulate current power reading"""
        if not self.connected:
            raise Exception("Device not connected")
        
        # Simulate realistic power consumption (0-150W with some randomness)
        self.power_watts = random.uniform(0, 150) if self.is_on else 0.0
        
        return {
            'power_watts': self.power_watts,
            'power_milliwatts': int(self.power_watts * 1000),
            'timestamp': datetime.now().isoformat()
        }
    
    async def get_energy_usage(self) -> dict:
        """Simulate energy usage"""
        if not self.connected:
            raise Exception("Device not connected")
        
        # Simulate accumulating energy (increment by small amounts)
        self.today_energy_wh += random.randint(1, 10)
        
        return {
            'today_energy_wh': self.today_energy_wh,
            'today_energy_kwh': self.today_energy_wh / 1000,
            'today_runtime_min': random.randint(0, 1440),
            'month_energy_wh': self.today_energy_wh * random.randint(20, 30),
            'month_energy_kwh': (self.today_energy_wh * random.randint(20, 30)) / 1000,
            'month_runtime_min': random.randint(0, 43200),
            'local_time': datetime.now().isoformat()
        }
    
    async def get_device_info(self) -> dict:
        """Simulate device info"""
        if not self.connected:
            raise Exception("Device not connected")
        
        return {
            'device_id': self.device_id,
            'model': 'P110-MOCK',
            'hardware_version': '1.0',
            'firmware_version': 'MOCK-1.0.0',
            'mac_address': 'AA-BB-CC-DD-EE-FF',
            'device_on': self.is_on,
            'rssi': -50
        }
    
    async def get_complete_snapshot(self) -> dict:
        """Get complete device snapshot"""
        if not self.connected:
            raise Exception("Device not connected")
        
        device_info = await self.get_device_info()
        current_power = await self.get_current_power()
        energy_usage = await self.get_energy_usage()
        
        return {
            'device_info': device_info,
            'current_power': current_power,
            'energy_usage': energy_usage,
            'snapshot_time': datetime.now().isoformat()
        }
    
    async def turn_on(self) -> bool:
        """Turn device on"""
        self.is_on = True
        logger.info("Mock device turned ON")
        return True
    
    async def turn_off(self) -> bool:
        """Turn device off"""
        self.is_on = False
        self.power_watts = 0.0
        logger.info("Mock device turned OFF")
        return True
    
    def set_power_watts(self, watts: float):
        """Manually set power consumption for testing"""
        self.power_watts = watts
        logger.info(f"Mock device power set to: {watts}W")
    
    def set_energy_wh(self, wh: int):
        """Manually set energy consumption for testing"""
        self.today_energy_wh = wh
        logger.info(f"Mock device energy set to: {wh}Wh")


class MockTapoMonitor:
    """Drop-in replacement for TapoMonitor using mock device"""
    
    def __init__(self, email: str, password: str, device_ip: str):
        self.email = email
        self.password = password  
        self.device_ip = device_ip
        self.device = MockTapoDevice(device_ip)
        logger.info(f"🎭 Using MOCK TAPO device (IP: {device_ip})")
    
    async def connect(self) -> bool:
        """Connect to mock device"""
        return await self.device.connect()
    
    async def get_current_power(self) -> dict:
        """Get current power from mock"""
        return await self.device.get_current_power()
    
    async def get_energy_usage(self) -> dict:
        """Get energy usage from mock"""
        return await self.device.get_energy_usage()
    
    async def get_device_info(self) -> dict:
        """Get device info from mock"""
        return await self.device.get_device_info()
    
    async def get_complete_snapshot(self) -> dict:
        """Get complete snapshot from mock"""
        return await self.device.get_complete_snapshot()
    
    async def turn_on(self) -> bool:
        """Turn mock device on"""
        return await self.device.turn_on()
    
    async def turn_off(self) -> bool:
        """Turn mock device off"""
        return await self.device.turn_off()


# Test the mock device
async def test_mock():
    """Test mock device functionality"""
    import json
    logging.basicConfig(level=logging.INFO)
    
    monitor = MockTapoMonitor("test@email.com", "password", "192.168.1.100")
    
    if await monitor.connect():
        print("\n📊 Complete Device Snapshot:")
        print("=" * 60)
        
        snapshot = await monitor.get_complete_snapshot()
        
        if snapshot:
            print(json.dumps(snapshot, indent=2))
            print("\n✅ Mock Tapo device working perfectly!")
        else:
            print("❌ Failed to get snapshot")
    else:
        print("❌ Connection failed")


if __name__ == "__main__":
    asyncio.run(test_mock())
