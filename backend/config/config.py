import os
from dotenv import load_dotenv

# Load environment variables
load_dotenv()


class Config:
    """Configuration for PowerGrid Oracle Service"""
    
    # Tapo Device
    TAPO_EMAIL = os.getenv('TAPO_EMAIL')
    TAPO_PASSWORD = os.getenv('TAPO_PASSWORD')
    TAPO_DEVICE_IP = os.getenv('TAPO_DEVICE_IP')
    
    # Blockchain
    SUBSTRATE_RPC_URL = os.getenv('SUBSTRATE_RPC_URL', 'ws://127.0.0.1:9944')
    DEVICE_OWNER_SEED = os.getenv('DEVICE_OWNER_SEED', '//Alice')
    
    # Contract Addresses
    TOKEN_CONTRACT = os.getenv('TOKEN_CONTRACT_ADDRESS')
    REGISTRY_CONTRACT = os.getenv('REGISTRY_CONTRACT_ADDRESS')
    GRID_SERVICE_CONTRACT = os.getenv('GRID_SERVICE_CONTRACT_ADDRESS')
    GOVERNANCE_CONTRACT = os.getenv('GOVERNANCE_CONTRACT_ADDRESS')
    
    # Service Settings
    MONITORING_INTERVAL = int(os.getenv('MONITORING_INTERVAL_SECONDS', 30))
    STAKE_AMOUNT = int(os.getenv('STAKE_AMOUNT', 2000000000000000000))
    
    # Device Metadata (for registration)
    DEVICE_METADATA = {
        'device_type': 'SmartPlug',
        'capacity_watts': 2000,
        'location': 'Delhi, India',
        'manufacturer': 'TP-Link',
        'model': 'Tapo P110',
        'firmware_version': '1.1.3',
        'installation_date': 1640995200000  # Unix timestamp in ms
    }
    
    @classmethod
    def validate(cls):
        """Validate required configuration"""
        required = [
            'TAPO_EMAIL', 'TAPO_PASSWORD', 'TAPO_DEVICE_IP',
            'REGISTRY_CONTRACT', 'GRID_SERVICE_CONTRACT'
        ]
        
        missing = []
        for field in required:
            if not getattr(cls, field, None):
                missing.append(field)
        
        if missing:
            raise ValueError(f"Missing required config: {', '.join(missing)}")
        
        return True


# Validate on import
try:
    Config.validate()
except ValueError as e:
    print(f"⚠️  Configuration incomplete: {e}")

