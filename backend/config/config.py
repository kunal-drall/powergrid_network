"""
PowerGrid Network Configuration Module

Loads configuration from environment variables for the oracle service.
"""

import os
from pathlib import Path
from dotenv import load_dotenv

# Load .env file from backend directory
env_path = Path(__file__).parent.parent / '.env'
load_dotenv(dotenv_path=env_path)


class Config:
    """Configuration class for PowerGrid Network backend"""
    
    # Blockchain Configuration
    SUBSTRATE_RPC_URL: str = os.getenv('SUBSTRATE_RPC_URL', 'ws://127.0.0.1:9944')
    DEVICE_OWNER_SEED: str = os.getenv('DEVICE_OWNER_SEED', '//Alice')
    
    # Contract Addresses (set after deployment)
    TOKEN_CONTRACT: str = os.getenv('TOKEN_CONTRACT_ADDRESS', '')
    REGISTRY_CONTRACT: str = os.getenv('REGISTRY_CONTRACT_ADDRESS', '')
    GRID_SERVICE_CONTRACT: str = os.getenv('GRID_SERVICE_CONTRACT_ADDRESS', '')
    GOVERNANCE_CONTRACT: str = os.getenv('GOVERNANCE_CONTRACT_ADDRESS', '')
    
    # Tapo Device Configuration
    TAPO_EMAIL: str = os.getenv('TAPO_EMAIL', '')
    TAPO_PASSWORD: str = os.getenv('TAPO_PASSWORD', '')
    TAPO_DEVICE_IP: str = os.getenv('TAPO_DEVICE_IP', '')
    
    # Mock device mode - set to 'true' to use mock device for testing
    USE_MOCK_DEVICE: bool = os.getenv('USE_MOCK_DEVICE', 'false').lower() == 'true'
    
    # Service Configuration
    MONITORING_INTERVAL_SECONDS: int = int(os.getenv('MONITORING_INTERVAL_SECONDS', '30'))
    
    # Stake amount in native units (12 decimals)
    # Default: 2 tokens = 2 * 10^12 native units
    STAKE_AMOUNT: int = int(os.getenv('STAKE_AMOUNT', '2000000000000'))
    
    # Token decimals
    SUBSTRATE_DECIMALS: int = 12
    SUBSTRATE_UNIT: int = 10 ** SUBSTRATE_DECIMALS
    
    @classmethod
    def get_tapo_monitor(cls):
        """Factory method to get appropriate TapoMonitor (mock or real)"""
        if cls.USE_MOCK_DEVICE:
            from mock_tapo_device import MockTapoMonitor
            print("🎭 Using MOCK Tapo device for testing")
            return MockTapoMonitor(
                cls.TAPO_EMAIL,
                cls.TAPO_PASSWORD,
                cls.TAPO_DEVICE_IP
            )
        else:
            from tapo_monitor import TapoMonitor
            print("🔌 Using REAL Tapo device")
            return TapoMonitor(
                cls.TAPO_EMAIL,
                cls.TAPO_PASSWORD,
                cls.TAPO_DEVICE_IP
            )
    
    
    @classmethod
    def validate(cls) -> bool:
        """Validate that required configuration is present"""
        errors = []
        
        # Check Tapo credentials
        if not cls.TAPO_EMAIL:
            errors.append("TAPO_EMAIL not set")
        if not cls.TAPO_PASSWORD:
            errors.append("TAPO_PASSWORD not set")
        if not cls.TAPO_DEVICE_IP:
            errors.append("TAPO_DEVICE_IP not set")
        
        # Check contract addresses
        if not cls.REGISTRY_CONTRACT:
            errors.append("REGISTRY_CONTRACT_ADDRESS not set")
        if not cls.GRID_SERVICE_CONTRACT:
            errors.append("GRID_SERVICE_CONTRACT_ADDRESS not set")
        if not cls.TOKEN_CONTRACT:
            errors.append("TOKEN_CONTRACT_ADDRESS not set")
        
        if errors:
            for error in errors:
                print(f"❌ Config error: {error}")
            return False
        
        print("✅ Configuration validated")
        return True
    
    @classmethod
    def tokens_to_native(cls, tokens: float) -> int:
        """Convert tokens to native units"""
        return int(tokens * cls.SUBSTRATE_UNIT)
    
    @classmethod
    def native_to_tokens(cls, native: int) -> float:
        """Convert native units to tokens"""
        return native / cls.SUBSTRATE_UNIT
    
    @classmethod
    def print_config(cls):
        """Print current configuration (hiding sensitive values)"""
        print("📋 Current Configuration:")
        print(f"   RPC URL: {cls.SUBSTRATE_RPC_URL}")
        print(f"   Seed: {cls.DEVICE_OWNER_SEED[:10]}..." if len(cls.DEVICE_OWNER_SEED) > 10 else f"   Seed: {cls.DEVICE_OWNER_SEED}")
        print(f"   Token Contract: {cls.TOKEN_CONTRACT[:20]}..." if cls.TOKEN_CONTRACT else "   Token Contract: NOT SET")
        print(f"   Registry Contract: {cls.REGISTRY_CONTRACT[:20]}..." if cls.REGISTRY_CONTRACT else "   Registry Contract: NOT SET")
        print(f"   Grid Service Contract: {cls.GRID_SERVICE_CONTRACT[:20]}..." if cls.GRID_SERVICE_CONTRACT else "   Grid Service Contract: NOT SET")
        print(f"   Tapo Email: {cls.TAPO_EMAIL[:5]}***" if cls.TAPO_EMAIL else "   Tapo Email: NOT SET")
        print(f"   Tapo Device IP: {cls.TAPO_DEVICE_IP}" if cls.TAPO_DEVICE_IP else "   Tapo Device IP: NOT SET")
        print(f"   Monitoring Interval: {cls.MONITORING_INTERVAL_SECONDS}s")


# For testing
if __name__ == "__main__":
    Config.print_config()
    Config.validate()
