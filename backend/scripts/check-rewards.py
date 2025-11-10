#!/usr/bin/env python3
"""Script to check token rewards and participation history"""

import sys
import os

# Add parent directories to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from blockchain_client import BlockchainClient
from config.config import Config

def main():
    print("=" * 60)
    print("💰 PowerGrid Network - Reward Checker")
    print("=" * 60)
    print()
    
    # Initialize client
    client = BlockchainClient(Config.SUBSTRATE_RPC_URL, Config.DEVICE_OWNER_SEED)
    
    if not client.connect():
        print("❌ Failed to connect to blockchain")
        return
    
    if not client.load_contracts(
        Config.TOKEN_CONTRACT,
        Config.REGISTRY_CONTRACT,
        Config.GRID_SERVICE_CONTRACT,
        Config.GOVERNANCE_CONTRACT
    ):
        print("❌ Failed to load contracts")
        return
    
    print("✅ Connected to blockchain")
    print()
    
    # Check token balance
    print("📊 Token Balance:")
    balance = client.get_token_balance()
    if balance > 0:
        balance_tokens = balance / 10**18
        print(f"   PWGD Balance: {balance_tokens:.4f} tokens")
        print(f"   Raw Balance: {balance} wei")
    else:
        print("   PWGD Balance: 0 tokens")
    print()
    
    # Check device registration
    print("📋 Device Status:")
    is_registered = client.is_device_registered()
    if is_registered:
        print("   ✅ Device is registered")
        reputation = client.get_device_reputation()
        print(f"   Reputation: {reputation}")
    else:
        print("   ❌ Device is not registered")
    print()
    
    # Check active events
    print("📢 Active Grid Events:")
    events = client.get_active_events()
    if events:
        print(f"   Found {len(events)} active event(s)")
        for idx, event in enumerate(events):
            if isinstance(event, (list, tuple)) and len(event) >= 2:
                event_id = event[0]
                print(f"   Event {event_id}: Active")
            else:
                print(f"   Event {idx}: Active")
    else:
        print("   No active events")
    print()
    
    # Account balance (for gas)
    print("⛽ Account Balance:")
    account_balance = client.get_account_balance()
    print(f"   Native tokens: {account_balance:.2f}")
    print()
    
    print("=" * 60)
    print("✅ Check complete!")
    print("=" * 60)

if __name__ == "__main__":
    main()

