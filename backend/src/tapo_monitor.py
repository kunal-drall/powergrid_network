import asyncio
from tapo import ApiClient
from datetime import datetime
import logging

logger = logging.getLogger(__name__)


class TapoMonitor:
    """Monitor Tapo P110 smart plug for energy data"""
    
    def __init__(self, email: str, password: str, device_ip: str):
        self.email = email
        self.password = password
        self.device_ip = device_ip
        self.client = None
        self.device = None
        
    async def connect(self):
        """Initialize connection to Tapo device"""
        try:
            logger.info(f"Connecting to Tapo P110 at {self.device_ip}...")
            self.client = ApiClient(self.email, self.password)
            self.device = await self.client.p110(self.device_ip)
            
            # Test connection
            device_info = await self.device.get_device_info()
            logger.info(f"✅ Connected to {device_info.model} (MAC: {device_info.mac})")
            
            return True
            
        except Exception as e:
            logger.error(f"❌ Failed to connect to Tapo device: {e}")
            return False
    
    async def get_current_power(self) -> dict:
        """Get current power consumption in watts"""
        try:
            power_result = await self.device.get_current_power()
            power_mw = power_result.current_power
            power_w = power_mw / 1000
            
            return {
                'power_watts': power_w,
                'power_milliwatts': power_mw,
                'timestamp': datetime.now().isoformat()
            }
            
        except Exception as e:
            logger.error(f"Error reading current power: {e}")
            return None
    
    async def get_energy_usage(self) -> dict:
        """Get accumulated energy usage"""
        try:
            energy_usage = await self.device.get_energy_usage()
            
            return {
                'today_energy_wh': energy_usage.today_energy,
                'today_energy_kwh': energy_usage.today_energy / 1000,
                'today_runtime_min': energy_usage.today_runtime,
                'month_energy_wh': energy_usage.month_energy,
                'month_energy_kwh': energy_usage.month_energy / 1000,
                'month_runtime_min': energy_usage.month_runtime,
                'local_time': energy_usage.local_time.isoformat()
            }
            
        except Exception as e:
            logger.error(f"Error reading energy usage: {e}")
            return None
    
    async def get_device_info(self) -> dict:
        """Get device information"""
        try:
            device_info = await self.device.get_device_info()
            
            return {
                'device_id': device_info.device_id,
                'model': device_info.model,
                'hardware_version': device_info.hw_ver,
                'firmware_version': device_info.fw_ver,
                'mac_address': device_info.mac,
                'device_on': device_info.device_on,
                'rssi': device_info.rssi
            }
            
        except Exception as e:
            logger.error(f"Error reading device info: {e}")
            return None
    
    async def get_complete_snapshot(self) -> dict:
        """Get complete device snapshot"""
        try:
            device_info = await self.get_device_info()
            current_power = await self.get_current_power()
            energy_usage = await self.get_energy_usage()
            
            return {
                'device_info': device_info,
                'current_power': current_power,
                'energy_usage': energy_usage,
                'snapshot_time': datetime.now().isoformat()
            }
            
        except Exception as e:
            logger.error(f"Error getting complete snapshot: {e}")
            return None
    
    async def turn_on(self):
        """Turn device on"""
        try:
            await self.device.on()
            logger.info("Device turned ON")
            return True
        except Exception as e:
            logger.error(f"Error turning device on: {e}")
            return False
    
    async def turn_off(self):
        """Turn device off"""
        try:
            await self.device.off()
            logger.info("Device turned OFF")
            return True
        except Exception as e:
            logger.error(f"Error turning device off: {e}")
            return False


# Test the monitor
async def test_monitor():
    """Test TapoMonitor functionality"""
    import sys
    import os
    sys.path.append(os.path.dirname(os.path.dirname(__file__)))
    from config.config import Config
    
    monitor = TapoMonitor(
        Config.TAPO_EMAIL,
        Config.TAPO_PASSWORD,
        Config.TAPO_DEVICE_IP
    )
    
    if await monitor.connect():
        print("\n📊 Complete Device Snapshot:")
        print("=" * 60)
        
        snapshot = await monitor.get_complete_snapshot()
        
        if snapshot:
            import json
            print(json.dumps(snapshot, indent=2))
            print("\n✅ Tapo Monitor working perfectly!")
        else:
            print("❌ Failed to get snapshot")
    else:
        print("❌ Connection failed")


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    asyncio.run(test_monitor())

