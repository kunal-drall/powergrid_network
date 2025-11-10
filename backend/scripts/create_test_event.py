#!/usr/bin/env python3
"""Script to create a test grid event for the oracle to participate in"""

import sys
import os

# Add parent directories to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from substrateinterface import SubstrateInterface, Keypair, ContractInstance
from config.config import Config

def create_test_event():
    """Create a test grid event for demo"""
    
    print("=" * 60)
    print("🎯 Creating Test Grid Event")
    print("=" * 60)
    
    # Connect
    try:
        substrate = SubstrateInterface(url=Config.SUBSTRATE_RPC_URL)
        keypair = Keypair.create_from_uri(Config.DEVICE_OWNER_SEED)
        
        print(f"✅ Connected as: {keypair.ss58_address}")
    except Exception as e:
        print(f"❌ Failed to connect: {e}")
        return
    
    # Load Grid Service contract
    try:
        # Use same path pattern as blockchain_client
        from pathlib import Path
        abi_dir = Path(__file__).parent.parent / 'config' / 'abis'
        abi_path = abi_dir / 'grid_service.json'
        
        if not abi_path.exists():
            print(f"❌ ABI file not found: {abi_path}")
            print("💡 Make sure contract ABIs are in backend/config/abis/")
            return
        
        grid_contract = ContractInstance.create_from_address(
            contract_address=Config.GRID_SERVICE_CONTRACT,
            metadata_file=str(abi_path),
            substrate=substrate
        )
        
        print(f"✅ Grid Service contract loaded: {Config.GRID_SERVICE_CONTRACT}")
    except Exception as e:
        print(f"❌ Failed to load contract: {e}")
        import traceback
        traceback.print_exc()
        return
    
    print("\n📢 Creating DemandResponse event...")
    print("   Duration: 60 minutes")
    print("   Compensation: 750 tokens per kWh (0.75 * 10^18 wei)")
    print("   Target: 100 kW reduction")
    print("")
    
    # Create DemandResponse event
    try:
        receipt = grid_contract.exec(
            keypair,
            'create_grid_event',
            args={
                'event_type': {'DemandResponse': None},
                'duration_minutes': 60,
                'compensation_rate': 750_000_000_000_000_000,  # 0.75 tokens
                'target_reduction_kw': 100
            },
            gas_limit={'ref_time': 10000000000, 'proof_size': 1000000}
        )
        
        if receipt.is_success:
            print("\n✅ Test event created successfully!")
            print(f"   Transaction: {receipt.extrinsic_hash}")
            print(f"   Block: {receipt.block_hash}")
            print("\n💡 The oracle will detect and participate in this event")
            print("   on its next monitoring cycle (within 30 seconds)")
        else:
            print(f"\n❌ Failed: {receipt.error_message}")
            if hasattr(receipt, 'error_data'):
                print(f"   Error data: {receipt.error_data}")
    except Exception as e:
        print(f"\n❌ Error creating event: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    create_test_event()

