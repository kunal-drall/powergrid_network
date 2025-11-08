from substrateinterface import SubstrateInterface, Keypair, ContractInstance
from substrateinterface.exceptions import SubstrateRequestException
import json
import logging
from pathlib import Path

logger = logging.getLogger(__name__)


class BlockchainClient:
    """Client for interacting with PowerGrid smart contracts"""
    
    def __init__(self, rpc_url: str, seed_phrase: str):
        self.rpc_url = rpc_url
        self.substrate = None
        self.keypair = None
        
        # Contract instances
        self.token_contract = None
        self.registry_contract = None
        self.grid_service_contract = None
        self.governance_contract = None
        
        # Initialize keypair
        self.keypair = Keypair.create_from_uri(seed_phrase)
        logger.info(f"Initialized keypair: {self.keypair.ss58_address}")
    
    def connect(self):
        """Connect to Substrate node"""
        try:
            logger.info(f"Connecting to {self.rpc_url}...")
            self.substrate = SubstrateInterface(
                url=self.rpc_url,
                type_registry_preset='substrate-node-template'  # or 'canvas' for ink! contracts
            )
            
            chain = self.substrate.chain
            logger.info(f"✅ Connected to {chain}")
            return True
            
        except Exception as e:
            logger.error(f"❌ Connection failed: {e}")
            return False
    
    def load_contracts(self, token_addr: str, registry_addr: str, 
                       grid_service_addr: str, governance_addr: str = None):
        """Load contract instances"""
        try:
            abi_dir = Path(__file__).parent.parent / 'config' / 'abis'
            
            # Load Token Contract
            with open(abi_dir / 'powergrid_token.json', 'r') as f:
                token_metadata = json.load(f)
            
            self.token_contract = ContractInstance.create_from_address(
                contract_address=token_addr,
                metadata_file=str(abi_dir / 'powergrid_token.json'),
                substrate=self.substrate
            )
            logger.info(f"✅ Token contract loaded: {token_addr}")
            
            # Load Registry Contract
            self.registry_contract = ContractInstance.create_from_address(
                contract_address=registry_addr,
                metadata_file=str(abi_dir / 'resource_registry.json'),
                substrate=self.substrate
            )
            logger.info(f"✅ Registry contract loaded: {registry_addr}")
            
            # Load Grid Service Contract
            self.grid_service_contract = ContractInstance.create_from_address(
                contract_address=grid_service_addr,
                metadata_file=str(abi_dir / 'grid_service.json'),
                substrate=self.substrate
            )
            logger.info(f"✅ Grid Service contract loaded: {grid_service_addr}")
            
            # Load Governance (optional)
            if governance_addr:
                self.governance_contract = ContractInstance.create_from_address(
                    contract_address=governance_addr,
                    metadata_file=str(abi_dir / 'governance.json'),
                    substrate=self.substrate
                )
                logger.info(f"✅ Governance contract loaded: {governance_addr}")
            
            return True
            
        except Exception as e:
            logger.error(f"❌ Failed to load contracts: {e}")
            return False
    
    # ===== REGISTRY CONTRACT METHODS =====
    
    def is_device_registered(self) -> bool:
        """Check if device is registered"""
        try:
            result = self.registry_contract.read(
                self.keypair,
                'is_device_registered',
                args={'account': self.keypair.ss58_address}
            )
            
            # Handle different response formats
            data = result.contract_result_data
            if isinstance(data, dict):
                # Handle Ok/Err format
                if 'Ok' in data:
                    return data['Ok']
                elif 'Err' in data:
                    return False
                else:
                    return bool(data)
            return bool(data)
            
        except Exception as e:
            logger.error(f"Error checking registration: {e}")
            return False
    
    def register_device(self, metadata: dict, stake_amount: int) -> bool:
        """Register device on blockchain"""
        try:
            logger.info(f"Registering device with stake: {stake_amount}")
            
            # Format metadata for contract
            device_metadata = {
                'device_type': {'SmartPlug': None},  # Enum format
                'capacity_watts': metadata['capacity_watts'],
                'location': metadata['location'],
                'manufacturer': metadata['manufacturer'],
                'model': metadata['model'],
                'firmware_version': metadata['firmware_version'],
                'installation_date': metadata['installation_date']
            }
            
            # Execute transaction
            receipt = self.registry_contract.exec(
                self.keypair,
                'register_device',
                args={'metadata': device_metadata},
                value=stake_amount,  # Stake amount
                gas_limit={'ref_time': 10000000000, 'proof_size': 1000000}
            )
            
            if receipt.is_success:
                logger.info(f"✅ Device registered successfully!")
                logger.info(f"   Transaction hash: {receipt.extrinsic_hash}")
                return True
            else:
                logger.error(f"❌ Registration failed: {receipt.error_message}")
                return False
                
        except Exception as e:
            logger.error(f"Error registering device: {e}")
            return False
    
    def get_device_reputation(self) -> int:
        """Get device reputation score"""
        try:
            result = self.registry_contract.read(
                self.keypair,
                'get_device_reputation',
                args={'account': self.keypair.ss58_address}
            )
            
            # Handle different response formats
            data = result.contract_result_data
            if isinstance(data, dict):
                # Handle Ok/Err format
                if 'Ok' in data:
                    value = data['Ok']
                    return int(value) if value is not None else 0
                elif 'Err' in data:
                    return 0
            elif data is None:
                return 0
            else:
                return int(data) if data else 0
            
        except Exception as e:
            logger.error(f"Error getting reputation: {e}")
            return 0
    
    # ===== GRID SERVICE CONTRACT METHODS =====
    
    def get_active_events(self) -> list:
        """Get list of active grid events"""
        try:
            result = self.grid_service_contract.read(
                self.keypair,
                'get_active_events'
            )
            
            # Handle different response formats
            data = result.contract_result_data
            if isinstance(data, list):
                # Handle ['Ok', <ink_type>] format
                if len(data) >= 2 and data[0] == 'Ok':
                    events_obj = data[1]
                    # Extract value from ink type
                    if hasattr(events_obj, 'value'):
                        events = events_obj.value
                        if isinstance(events, list):
                            logger.info(f"Found {len(events)} active events")
                            return events
                    return []
                # Regular list
                logger.info(f"Found {len(data)} active events")
                return data
            elif isinstance(data, dict):
                # Handle Ok/Err format
                if 'Ok' in data:
                    events = data['Ok']
                    # Extract from ink type if needed
                    if hasattr(events, 'value'):
                        events = events.value
                    if isinstance(events, list):
                        logger.info(f"Found {len(events)} active events")
                        return events
                    else:
                        # Might be a tuple or other structure
                        return list(events) if events else []
                elif 'Err' in data:
                    return []
            else:
                # Try to convert to list
                try:
                    # Extract from ink type if needed
                    if hasattr(data, 'value'):
                        events = data.value
                    else:
                        events = data
                    events_list = list(events) if events else []
                    logger.info(f"Found {len(events_list)} active events")
                    return events_list
                except:
                    return []
            
            return []
            
        except Exception as e:
            logger.error(f"Error getting active events: {e}")
            return []
    
    def participate_in_event(self, event_id: int, energy_contribution_wh: int) -> bool:
        """Participate in a grid event"""
        try:
            logger.info(f"Participating in event {event_id} with {energy_contribution_wh} Wh")
            
            receipt = self.grid_service_contract.exec(
                self.keypair,
                'participate_in_event',
                args={
                    'event_id': event_id,
                    'energy_reduction_wh': energy_contribution_wh
                },
                gas_limit={'ref_time': 10000000000, 'proof_size': 1000000}
            )
            
            if receipt.is_success:
                logger.info(f"✅ Participation recorded!")
                return True
            else:
                logger.error(f"❌ Participation failed: {receipt.error_message}")
                return False
                
        except Exception as e:
            logger.error(f"Error participating in event: {e}")
            return False
    
    # ===== TOKEN CONTRACT METHODS =====
    
    def get_token_balance(self) -> int:
        """Get PWGD token balance"""
        try:
            result = self.token_contract.read(
                self.keypair,
                'balance_of',
                args={'owner': self.keypair.ss58_address}
            )
            
            # Handle different response formats
            data = result.contract_result_data
            
            # Try to get value from ink type (value attribute contains the actual dict)
            if hasattr(data, 'value'):
                data = data.value
            
            # Try dict-like access
            try:
                if isinstance(data, dict) and 'Ok' in data:
                    balance = data['Ok']
                    if isinstance(balance, int):
                        logger.info(f"Token balance: {balance}")
                        return balance
                elif isinstance(data, dict) and 'Err' in data:
                    logger.warning(f"Error getting balance: {data['Err']}")
                    return 0
            except (TypeError, KeyError):
                pass
            
            # Try string representation parsing (for ink types that show as dicts)
            data_str = str(data)
            if "'Ok':" in data_str or '"Ok":' in data_str:
                import re
                # Extract the number after Ok
                match = re.search(r"['\"]Ok['\"]:\s*(\d+)", data_str)
                if match:
                    balance_int = int(match.group(1))
                    logger.info(f"Token balance (parsed): {balance_int}")
                    return balance_int
            
            # Try direct int conversion
            if isinstance(data, (int, str)):
                balance_int = int(data) if isinstance(data, str) else data
                logger.info(f"Token balance: {balance_int}")
                return balance_int
            
            # Try to extract int from complex types (ink types)
            try:
                if hasattr(data, 'value'):
                    return int(data.value)
                if hasattr(data, '__int__'):
                    return int(data)
            except Exception as e:
                logger.debug(f"Failed to extract balance: {e}")
            
            logger.warning(f"Unexpected balance format: {data} (type: {type(data)})")
            return 0
            
        except Exception as e:
            logger.error(f"Error getting token balance: {e}")
            return 0
    
    # ===== UTILITY METHODS =====
    
    def get_account_balance(self) -> float:
        """Get native token balance (for gas fees)"""
        try:
            result = self.substrate.query(
                'System',
                'Account',
                [self.keypair.ss58_address]
            )
            
            balance = result.value['data']['free']
            balance_tokens = balance / 10**12  # Convert to tokens
            
            logger.info(f"Account balance: {balance_tokens} tokens")
            return balance_tokens
            
        except Exception as e:
            logger.error(f"Error getting account balance: {e}")
            return 0


# Test the blockchain client
def test_client():
    """Test BlockchainClient functionality"""
    import sys
    import os
    sys.path.append(os.path.dirname(os.path.dirname(__file__)))
    from config.config import Config
    import logging
    
    logging.basicConfig(level=logging.INFO)
    
    # Initialize client
    client = BlockchainClient(
        Config.SUBSTRATE_RPC_URL,
        Config.DEVICE_OWNER_SEED
    )
    
    # Connect
    if client.connect():
        print("✅ Connected to blockchain")
        
        # Check account balance
        balance = client.get_account_balance()
        print(f"Account balance: {balance} tokens")
        
        # Load contracts (need addresses first!)
        if Config.REGISTRY_CONTRACT and Config.GRID_SERVICE_CONTRACT:
            client.load_contracts(
                Config.TOKEN_CONTRACT,
                Config.REGISTRY_CONTRACT,
                Config.GRID_SERVICE_CONTRACT
            )
            
            # Check if device is registered
            is_registered = client.is_device_registered()
            print(f"Device registered: {is_registered}")
        else:
            print("⚠️  Contract addresses not configured yet")
    else:
        print("❌ Connection failed")


if __name__ == "__main__":
    test_client()

