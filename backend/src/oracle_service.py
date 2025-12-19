import asyncio
import logging
from datetime import datetime
import sys
import os

# Add parent directory to path for imports
sys.path.append(os.path.dirname(os.path.dirname(__file__)))

from tapo_monitor import TapoMonitor
from mock_tapo_device import MockTapoDevice
from blockchain_client import BlockchainClient
from config.config import Config

# Configure logging
log_dir = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'logs')
os.makedirs(log_dir, exist_ok=True)

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler(os.path.join(log_dir, 'oracle.log')),
        logging.StreamHandler()
    ]
)

logger = logging.getLogger(__name__)


class PowerGridOracle:
    """Main Oracle Service for PowerGrid Network"""
    
    def __init__(self):
        self.tapo_monitor = None
        self.blockchain_client = None
        self.is_running = False
        self.device_registered = False
        
    async def initialize(self):
        """Initialize all components"""
        logger.info("=" * 60)
        logger.info("🚀 PowerGrid Oracle Service Starting...")
        logger.info("=" * 60)
        
        # Validate configuration
        try:
            Config.validate()
            logger.info("✅ Configuration validated")
        except ValueError as e:
            logger.error(f"❌ Configuration error: {e}")
            return False
        
        # Initialize Tapo Monitor (real or mock)
        logger.info("\n📡 Initializing Tapo Monitor (%s mode)...", "mock" if Config.USE_MOCK_TAPO else "hardware")
        if Config.USE_MOCK_TAPO:
            self.tapo_monitor = MockTapoDevice(profile=Config.MOCK_TAPO_PROFILE)
            # Mock connect always succeeds
            await self.tapo_monitor.connect()
        else:
            self.tapo_monitor = TapoMonitor(
                Config.TAPO_EMAIL,
                Config.TAPO_PASSWORD,
                Config.TAPO_DEVICE_IP
            )
            if not await self.tapo_monitor.connect():
                logger.warning("⚠️  Failed to connect to Tapo device - will retry in monitoring loop")
                # Don't fail completely, allow retries
        
        # Initialize Blockchain Client
        logger.info("\n⛓️  Initializing Blockchain Client...")
        self.blockchain_client = BlockchainClient(
            Config.SUBSTRATE_RPC_URL,
            Config.DEVICE_OWNER_SEED
        )
        
        if not self.blockchain_client.connect():
            logger.error("❌ Failed to connect to blockchain")
            return False
        
        # Load contracts
        if not self.blockchain_client.load_contracts(
            Config.TOKEN_CONTRACT,
            Config.REGISTRY_CONTRACT,
            Config.GRID_SERVICE_CONTRACT,
            Config.GOVERNANCE_CONTRACT
        ):
            logger.error("❌ Failed to load contracts")
            return False
        
        # Check account balance
        balance = self.blockchain_client.get_account_balance()
        if balance < 10:  # Need at least 10 tokens for gas
            logger.warning(f"⚠️  Low account balance: {balance} tokens")
        else:
            logger.info(f"✅ Account balance: {balance:.2f} tokens")
        
        logger.info("\n✅ Oracle initialized successfully!")
        return True
    
    async def ensure_device_registered(self):
        """Ensure device is registered on blockchain"""
        logger.info("\n🔍 Checking device registration...")
        
        is_registered = self.blockchain_client.is_device_registered()
        
        if is_registered:
            logger.info("✅ Device already registered")
            self.device_registered = True
            
            # Get reputation
            reputation = self.blockchain_client.get_device_reputation()
            logger.info(f"📊 Current reputation: {reputation}")
            return True
        
        # Device not registered - register it now
        logger.info("📝 Device not registered. Registering now...")
        
        success = self.blockchain_client.register_device(
            Config.DEVICE_METADATA,
            Config.STAKE_AMOUNT
        )
        
        if success:
            self.device_registered = True
            logger.info("✅ Device registered successfully!")
            return True
        else:
            logger.error("❌ Device registration failed")
            return False
    
    async def check_and_participate_in_events(self, energy_data: dict):
        """Check for active events and participate"""
        try:
            # Get active events
            active_events = self.blockchain_client.get_active_events()
            
            if not active_events:
                logger.debug("No active grid events")
                return
            
            logger.info(f"📢 Found {len(active_events)} active event(s)")
            
            # For each active event
            for idx, event in enumerate(active_events):
                event_id = idx
                event_data = event

                # Handle tuple/list format: (event_id, GridEvent)
                if isinstance(event, (list, tuple)):
                    if len(event) >= 2:
                        event_id = event[0]
                        event_data = event[1]
                    elif len(event) == 1:
                        event_data = event[0]
                elif isinstance(event, dict):
                    event_id = event.get('event_id', event.get('id', event.get('key', idx)))
                    event_data = event.get('event', event.get('value', event))

                # Unwrap ink! Ok/Err structures if present
                if isinstance(event_data, dict) and 'Ok' in event_data:
                    event_data = event_data['Ok']
                if isinstance(event_data, dict) and 'Err' in event_data:
                    logger.warning(f"Event {event_id} returned Err: {event_data['Err']}")
                    continue

                # Normalize event_id into int when possible
                try:
                    event_id = int(event_id)
                except (TypeError, ValueError):
                    event_id = idx

                # Extract event info from GridEvent struct
                # GridEvent has: event_type, duration_minutes, base_compensation_rate, target_reduction_kw, ...
                event_type = 'Unknown'
                target_reduction = 0
                compensation_rate = 0
                
                # Handle different data structures
                if isinstance(event_data, dict):
                    event_type = event_data.get('event_type', 'Unknown')
                    target_reduction = event_data.get('target_reduction_kw', event_data.get('target_reduction', 0))
                    compensation_rate = event_data.get('base_compensation_rate', event_data.get('compensation_rate', 0))
                elif hasattr(event_data, '__dict__'):
                    event_type = getattr(event_data, 'event_type', 'Unknown')
                    target_reduction = getattr(event_data, 'target_reduction_kw', getattr(event_data, 'target_reduction', 0))
                    compensation_rate = getattr(event_data, 'base_compensation_rate', getattr(event_data, 'compensation_rate', 0))
                elif isinstance(event_data, (list, tuple)):
                    seq = list(event_data)
                    event_type = seq[0] if len(seq) > 0 else 'Unknown'
                    compensation_rate = seq[2] if len(seq) > 2 else (seq[1] if len(seq) > 1 else 0)
                    target_reduction = seq[3] if len(seq) > 3 else (seq[2] if len(seq) > 2 else 0)
                
                # Handle event_type enum (could be dict like {'DemandResponse': None} or string)
                if isinstance(event_type, dict):
                    event_type_str = list(event_type.keys())[0] if event_type else 'Unknown'
                else:
                    event_type_str = str(event_type)
                
                # Convert compensation rate from wei to tokens (18 decimals)
                compensation_tokens = compensation_rate / 10**18 if compensation_rate else 0
                
                logger.info(f"\n🎯 Event {event_id}: {event_type_str}")
                logger.info(f"   Target reduction: {target_reduction} kW")
                logger.info(f"   Compensation rate: {compensation_tokens:.4f} tokens/kWh ({compensation_rate} wei)")
                
                # Calculate energy contribution (today's energy in Wh)
                try:
                    energy_contribution = energy_data.get('energy_usage', {}).get('today_energy_wh', 0) if energy_data else 0
                except (KeyError, TypeError):
                    energy_contribution = 0
                
                if energy_contribution > 0:
                    # Participate in event
                    success = self.blockchain_client.participate_in_event(
                        event_id,
                        energy_contribution
                    )
                    
                    if success:
                        logger.info(f"✅ Participated with {energy_contribution} Wh")
                    else:
                        logger.error(f"❌ Participation failed for event {event_id}")
                else:
                    logger.info("⚠️  No energy contribution to report yet")
                    
        except Exception as e:
            logger.error(f"Error in event participation: {e}")
    
    async def monitoring_loop(self):
        """Main monitoring loop"""
        logger.info("\n" + "=" * 60)
        logger.info("🔄 Starting monitoring loop...")
        logger.info(f"⏱️  Interval: {Config.MONITORING_INTERVAL} seconds")
        logger.info("=" * 60)
        
        iteration = 0
        
        while self.is_running:
            try:
                iteration += 1
                logger.info(f"\n{'='*60}")
                logger.info(f"📊 Monitoring Iteration #{iteration}")
                logger.info(f"🕐 {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
                logger.info(f"{'='*60}")
                
                # 1. Get device snapshot (with retry logic)
                logger.info("1️⃣  Reading Tapo device data...")
                snapshot = None

                monitor_has_device_attr = hasattr(self.tapo_monitor, "device")
                monitor_ready = True

                # Retry connection if needed for real hardware monitor
                if monitor_has_device_attr and not getattr(self.tapo_monitor, "device"):
                    logger.info("   Reconnecting to Tapo device...")
                    await self.tapo_monitor.connect()
                    monitor_ready = bool(getattr(self.tapo_monitor, "device"))

                if monitor_ready:
                    snapshot = await self.tapo_monitor.get_complete_snapshot()

                if not snapshot:
                    logger.warning("⚠️  Failed to get device snapshot - will retry next iteration")
                    # Try to reconnect for hardware path
                    if monitor_has_device_attr and not getattr(self.tapo_monitor, "device"):
                        logger.info("   Attempting to reconnect to Tapo device...")
                        await self.tapo_monitor.connect()
                    await asyncio.sleep(Config.MONITORING_INTERVAL)
                    continue
                
                # Log current stats
                try:
                    current_power = snapshot.get('current_power', {}).get('power_watts', 0.0) if snapshot else 0.0
                    today_energy = snapshot.get('energy_usage', {}).get('today_energy_kwh', 0.0) if snapshot else 0.0
                except (KeyError, TypeError) as e:
                    logger.error(f"Error parsing snapshot data: {e}")
                    logger.warning("⚠️  Invalid snapshot data - will retry next iteration")
                    await asyncio.sleep(Config.MONITORING_INTERVAL)
                    continue
                
                logger.info(f"⚡ Current Power: {current_power:.2f} W")
                logger.info(f"📈 Today's Energy: {today_energy:.3f} kWh")
                
                # 2. Check device registration (once)
                if not self.device_registered:
                    logger.info("\n2️⃣  Checking device registration...")
                    await self.ensure_device_registered()
                
                # 3. Check for active grid events
                logger.info("\n3️⃣  Checking for grid events...")
                await self.check_and_participate_in_events(snapshot)
                
                # 4. Check token balance
                logger.info("\n4️⃣  Checking rewards...")
                token_balance = self.blockchain_client.get_token_balance()
                if token_balance > 0:
                    balance_tokens = token_balance / 10**18
                    logger.info(f"💰 PWGD Balance: {balance_tokens:.4f} tokens")
                else:
                    logger.info(f"💰 PWGD Balance: 0 tokens")
                
                # Wait before next iteration
                logger.info(f"\n⏳ Waiting {Config.MONITORING_INTERVAL} seconds...")
                await asyncio.sleep(Config.MONITORING_INTERVAL)
                
            except KeyboardInterrupt:
                logger.info("\n⚠️  Interrupt received, shutting down...")
                break
                
            except Exception as e:
                logger.error(f"❌ Error in monitoring loop: {e}", exc_info=True)
                await asyncio.sleep(Config.MONITORING_INTERVAL)
    
    async def start(self):
        """Start the oracle service"""
        if not await self.initialize():
            logger.error("❌ Initialization failed. Exiting.")
            return
        
        self.is_running = True
        
        try:
            await self.monitoring_loop()
        finally:
            self.is_running = False
            logger.info("\n" + "=" * 60)
            logger.info("👋 PowerGrid Oracle Service Stopped")
            logger.info("=" * 60)
    
    async def stop(self):
        """Stop the oracle service"""
        self.is_running = False


# Main entry point
async def main():
    oracle = PowerGridOracle()
    
    try:
        await oracle.start()
    except KeyboardInterrupt:
        logger.info("\n⚠️  Shutting down gracefully...")
        await oracle.stop()


if __name__ == "__main__":
    asyncio.run(main())

