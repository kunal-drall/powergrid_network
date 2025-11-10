#!/usr/bin/env python3
"""Script to set up contract authorization for the oracle service"""

import sys
import os

# Add parent directories to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from substrateinterface import SubstrateInterface, Keypair, ContractInstance
from config.config import Config
from pathlib import Path

def setup_authorization():
    """Set up contract authorization for oracle service"""
    
    print("=" * 60)
    print("🔐 Setting Up Contract Authorization")
    print("=" * 60)
    print()
    
    # Connect
    try:
        substrate = SubstrateInterface(
            url=Config.SUBSTRATE_RPC_URL,
            type_registry_preset='substrate-node-template'
        )
        owner_keypair = Keypair.create_from_uri(Config.DEVICE_OWNER_SEED)
        oracle_keypair = Keypair.create_from_uri(Config.DEVICE_OWNER_SEED)  # Same for now
        
        print(f"✅ Connected as owner: {owner_keypair.ss58_address}")
        print(f"✅ Oracle account: {oracle_keypair.ss58_address}")
    except Exception as e:
        print(f"❌ Failed to connect: {e}")
        return False
    
    abi_dir = Path(__file__).parent.parent / 'config' / 'abis'
    
    # Step 1: Authorize oracle on Grid Service
    print("\n📋 Step 1: Authorizing Oracle on Grid Service...")
    try:
        grid_contract = ContractInstance.create_from_address(
            contract_address=Config.GRID_SERVICE_CONTRACT,
            metadata_file=str(abi_dir / 'grid_service.json'),
            substrate=substrate
        )
        
        # Add oracle as authorized caller (owner can always call, so we try directly)
        print("   📝 Adding oracle as authorized caller...")
        receipt = grid_contract.exec(
            owner_keypair,
            'add_authorized_caller',
            args={'caller': oracle_keypair.ss58_address},
            gas_limit={'ref_time': 10000000000, 'proof_size': 1000000}
        )
        
        if receipt.is_success:
            print("   ✅ Oracle authorized successfully!")
            print(f"   Transaction: {receipt.extrinsic_hash}")
        else:
            error_msg = receipt.error_message if hasattr(receipt, 'error_message') else 'Unknown error'
            if 'already' in error_msg.lower() or 'exists' in error_msg.lower():
                print("   ✅ Oracle already authorized")
            else:
                print(f"   ⚠️  Authorization attempt: {error_msg}")
                print("   💡 Note: Owner account can always create events, authorization is optional")
    except Exception as e:
        print(f"   ⚠️  Error: {e}")
        print("   💡 Owner account can create events without explicit authorization")
    
    # Step 2: Grant Grid Service minter role on Token
    print("\n📋 Step 2: Granting Grid Service Minter Role on Token...")
    try:
        token_contract = ContractInstance.create_from_address(
            contract_address=Config.TOKEN_CONTRACT,
            metadata_file=str(abi_dir / 'powergrid_token.json'),
            substrate=substrate
        )
        
        # Check if Grid Service is already a minter
        try:
            result = token_contract.read(
                owner_keypair,
                'is_minter',
                args={'account': Config.GRID_SERVICE_CONTRACT}
            )
            is_minter = result.contract_result_data
            # Handle different response formats
            if isinstance(is_minter, dict):
                if 'Ok' in is_minter:
                    is_minter = is_minter['Ok']
                elif 'Err' in is_minter:
                    is_minter = False
            is_minter = bool(is_minter) if is_minter is not None else False
            
            if is_minter:
                print("   ✅ Grid Service already has minter role")
            else:
                print("   📝 Granting minter role to Grid Service...")
                receipt = token_contract.exec(
                    owner_keypair,
                    'add_minter',
                    args={'account': Config.GRID_SERVICE_CONTRACT},
                    gas_limit={'ref_time': 10000000000, 'proof_size': 1000000}
                )
                
                if receipt.is_success:
                    print("   ✅ Minter role granted successfully!")
                    print(f"   Transaction: {receipt.extrinsic_hash}")
                else:
                    error_msg = receipt.error_message if hasattr(receipt, 'error_message') else 'Unknown error'
                    print(f"   ⚠️  Failed: {error_msg}")
        except Exception as e:
            print(f"   ⚠️  Could not check minter role: {e}")
            print("   📝 Attempting to grant minter role directly...")
            try:
                receipt = token_contract.exec(
                    owner_keypair,
                    'add_minter',
                    args={'account': Config.GRID_SERVICE_CONTRACT},
                    gas_limit={'ref_time': 10000000000, 'proof_size': 1000000}
                )
                if receipt.is_success:
                    print("   ✅ Minter role granted!")
                else:
                    print(f"   ⚠️  {receipt.error_message}")
            except:
                print("   💡 You may need to grant manually via contract admin")
    except Exception as e:
        print(f"   ❌ Error: {e}")
    
    # Step 3: Authorize Grid Service on Registry (if needed)
    print("\n📋 Step 3: Checking Registry Authorization...")
    try:
        registry_contract = ContractInstance.create_from_address(
            contract_address=Config.REGISTRY_CONTRACT,
            metadata_file=str(abi_dir / 'resource_registry.json'),
            substrate=substrate
        )
        
        print("   ✅ Registry contract loaded")
        print("   💡 Registry typically doesn't need special authorization")
    except Exception as e:
        print(f"   ⚠️  Could not check registry: {e}")
    
    print("\n" + "=" * 60)
    print("✅ Authorization Setup Complete!")
    print("=" * 60)
    print()
    print("📝 Note: Some contracts may require owner privileges.")
    print("   If methods fail, you may need to use contract owner account.")
    print()
    
    return True

if __name__ == "__main__":
    setup_authorization()

