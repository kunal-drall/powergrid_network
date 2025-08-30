#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod resource_registry {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;
    use powergrid_shared::{DeviceMetadata, Device, ink_account_to_bytes};

    /// The ResourceRegistry contract
    #[ink(storage)]
    pub struct ResourceRegistry {
        /// Simple reentrancy flag
        entered: bool,
        /// Pause flag
        paused: bool,
        /// Mapping from AccountId to Device info (using [u8; 32] as key)
        devices: Mapping<[u8; 32], Device>,
        /// Minimum stake required for device registration
        min_stake: Balance,
        /// Owner of the contract (using ink! AccountId for env() compatibility)
        owner: Option<AccountId>,
        /// Total number of registered devices
        device_count: u64,
    /// Authorized callers map
    authorized_callers: Mapping<AccountId, bool>,
    /// Reputation threshold for eligibility (governance managed)
    reputation_threshold: u32,
    /// Governance contract (optional) that can manage roles/params
    governance_address: Option<AccountId>,
    }

    /// Events emitted by the contract
    #[ink(event)]
    pub struct DeviceRegistered {
        #[ink(topic)]
        account: AccountId,
        stake: Balance,
        reputation: u32,
    }

    #[ink(event)]
    pub struct StakeIncreased {
        #[ink(topic)]
        account: AccountId,
        additional_stake: Balance,
        total_stake: Balance,
    }

    #[ink(event)]
    pub struct StakeWithdrawn {
        #[ink(topic)]
        account: AccountId,
        amount: Balance,
        remaining_stake: Balance,
    }

    #[ink(event)]
    pub struct StakeSlashed {
        #[ink(topic)]
        account: AccountId,
        amount: Balance,
        remaining_stake: Balance,
        reason: String,
    }

    #[ink(event)]
    pub struct DeviceDeactivated {
        #[ink(topic)]
        account: AccountId,
        reason: String,
    }

    #[ink(event)]
    pub struct ReputationUpdated {
        #[ink(topic)]
        account: AccountId,
        old_reputation: u32,
        new_reputation: u32,
    }

    #[ink(event)]
    pub struct DeviceUpdated {
        #[ink(topic)]
        account: AccountId,
        version: u32,
        timestamp: u64,
    }

    impl ResourceRegistry {
        /// Constructor
        #[ink(constructor)]
        pub fn new(min_stake: Balance) -> Self {
            Self {
                devices: Mapping::default(),
                min_stake,
                owner: Some(Self::env().caller()),
                device_count: 0,
                authorized_callers: Mapping::default(),
                reputation_threshold: 50,
                governance_address: Some(Self::env().caller()),
                entered: false,
                paused: false,
            }
        }

        /// Register a new device with stake
        #[ink(message, payable)]
        pub fn register_device(&mut self, metadata: DeviceMetadata) -> Result<(), String> {
            if self.entered { return Err("Reentrancy".into()); }
            self.entered = true;
            if self.paused { self.entered = false; return Err("Paused".into()); }
            let caller = self.env().caller();
            let caller_bytes = ink_account_to_bytes(caller);
            let stake: Balance = self.env().transferred_value();
            
            if stake < self.min_stake {
                return Err("Insufficient stake amount".into());
            }
            
            if self.devices.contains(caller_bytes) {
                return Err("Device already registered".into());
            }

            let now = self.env().block_timestamp();
            let device = Device {
                metadata,
                stake,
                reputation: 100, // Initial reputation
                total_energy_contributed: 0,
                successful_events: 0,
                failed_events: 0,
                last_activity: now,
                active: true,
                version: 1,
                last_updated: now,
            };

            self.devices.insert(caller_bytes, &device);
            self.device_count = self.device_count.saturating_add(1);

            self.env().emit_event(DeviceRegistered {
                account: caller,
                stake,
                reputation: device.reputation,
            });
            self.entered = false;
            Ok(())
        }

        /// Increase stake for existing device
        #[ink(message, payable)]
        pub fn increase_stake(&mut self) -> Result<(), String> {
            if self.entered { return Err("Reentrancy".into()); }
            self.entered = true;
            if self.paused { self.entered = false; return Err("Paused".into()); }
            let caller = self.env().caller();
            let caller_bytes = ink_account_to_bytes(caller);
            let additional_stake: Balance = self.env().transferred_value();
            
            let mut device = self.devices.get(caller_bytes)
                .ok_or("Device not registered")?;
            
            device.stake = device.stake.saturating_add(additional_stake);
            self.devices.insert(caller_bytes, &device);

            self.env().emit_event(StakeIncreased {
                account: caller,
                additional_stake,
                total_stake: device.stake,
            });
            self.entered = false;
            Ok(())
        }

        /// Get device information
        #[ink(message)]
        pub fn get_device(&self, account: AccountId) -> Option<Device> {
            let account_bytes = ink_account_to_bytes(account);
            self.devices.get(account_bytes)
        }

        /// Check if device is registered
        #[ink(message)]
        pub fn is_device_registered(&self, account: AccountId) -> bool {
            let account_bytes = ink_account_to_bytes(account);
            self.devices.contains(account_bytes)
        }

        /// Get device reputation
        #[ink(message)]
        pub fn get_device_reputation(&self, account: AccountId) -> Option<u32> {
            let account_bytes = ink_account_to_bytes(account);
            self.devices.get(account_bytes).map(|device| device.reputation)
        }

        /// Get device count
        #[ink(message)]
        pub fn get_device_count(&self) -> u64 {
            self.device_count
        }

        /// Get minimum stake
        #[ink(message)]
        pub fn get_min_stake(&self) -> Balance {
            self.min_stake
        }

        /// Update device performance (authorized callers only)
        #[ink(message)]
        pub fn update_device_performance(&mut self, account: AccountId, energy_contributed: u64, success: bool) -> Result<(), String> {
            if self.ensure_authorized().is_err() {
                return Err("Unauthorized caller".into());
            }
            
            let account_bytes = ink_account_to_bytes(account);
            let mut device = self.devices.get(account_bytes)
                .ok_or("Device not registered")?;
            
            device.total_energy_contributed = device.total_energy_contributed.saturating_add(energy_contributed);
            if success {
                device.successful_events = device.successful_events.saturating_add(1);
            } else {
                device.failed_events = device.failed_events.saturating_add(1);
            }
            
            let old_reputation = device.reputation;
            device.reputation = self.calculate_performance_score(&device);
            device.last_activity = self.env().block_timestamp();
            device.version = device.version.saturating_add(1);
            device.last_updated = device.last_activity;
            
            self.devices.insert(account_bytes, &device);

            self.env().emit_event(ReputationUpdated {
                account,
                old_reputation,
                new_reputation: device.reputation,
            });
            self.env().emit_event(DeviceUpdated { account, version: device.version, timestamp: device.last_updated });

            Ok(())
        }

        /// Performance scoring algorithm
        fn calculate_performance_score(&self, device: &Device) -> u32 {
            let total_events = device.successful_events.saturating_add(device.failed_events);
            if total_events == 0 {
                return 100; // Default score for new devices
            }

            let success_rate = device.successful_events
                .saturating_mul(100)
                .checked_div(total_events)
                .unwrap_or(0);
            
            let energy_factor = device.total_energy_contributed
                .checked_div(1000)
                .unwrap_or(0)
                .min(50); // Up to 50 points for energy
            
            let base_score = success_rate.saturating_add(energy_factor as u32);
            base_score.clamp(1, 100) // Keep between 1-100
        }

        /// Update minimum stake (owner only)
        #[ink(message)]
        pub fn update_min_stake(&mut self, new_min_stake: Balance) -> Result<(), String> {
            let sender = self.env().caller();
            if Some(sender) != self.owner && Some(sender) != self.governance_address {
                return Err("Only owner/governance can update minimum stake".into());
            }
            
            self.min_stake = new_min_stake;
            Ok(())
        }

        /// Update reputation threshold (owner only)
        #[ink(message)]
        pub fn update_reputation_threshold(&mut self, new_threshold: u32) -> Result<(), String> {
            let sender = self.env().caller();
            if Some(sender) != self.owner && Some(sender) != self.governance_address {
                return Err("Only owner/governance can update reputation threshold".into());
            }
            self.reputation_threshold = new_threshold;
            Ok(())
        }

        /// Get reputation threshold
        #[ink(message)]
        pub fn get_reputation_threshold(&self) -> u32 {
            self.reputation_threshold
        }

        /// Add authorized caller (owner only)
        #[ink(message)]
        pub fn add_authorized_caller(&mut self, caller: AccountId) -> Result<(), String> {
            let sender = self.env().caller();
            if Some(sender) != self.owner && Some(sender) != self.governance_address {
                return Err("Only owner/governance can add authorized callers".into());
            }
            self.authorized_callers.insert(caller, &true);
            Ok(())
        }

        /// Remove authorized caller (owner only)
        #[ink(message)]
        pub fn remove_authorized_caller(&mut self, caller: AccountId) -> Result<(), String> {
            let sender = self.env().caller();
            if Some(sender) != self.owner && Some(sender) != self.governance_address {
                return Err("Only owner/governance can remove authorized callers".into());
            }
            self.authorized_callers.remove(caller);
            Ok(())
        }

        /// Set governance address (owner only)
        #[ink(message)]
        pub fn set_governance_address(&mut self, addr: AccountId) -> Result<(), String> {
            if Some(self.env().caller()) != self.owner {
                return Err("Only owner can set governance address".into());
            }
            self.governance_address = Some(addr);
            Ok(())
        }

        /// Deactivate a device (owner only)
        #[ink(message)]
        pub fn deactivate_device(&mut self, account: AccountId, reason: String) -> Result<(), String> {
            if Some(self.env().caller()) != self.owner {
                return Err("Only owner can deactivate devices".into());
            }

            let account_bytes = ink_account_to_bytes(account);
            let mut device = self.devices.get(account_bytes)
                .ok_or("Device not registered")?;
            
            device.active = false;
            self.devices.insert(account_bytes, &device);

            self.env().emit_event(DeviceDeactivated {
                account,
                reason,
            });

            Ok(())
        }

        /// Reactivate a device (owner only)
        #[ink(message)]
        pub fn reactivate_device(&mut self, account: AccountId) -> Result<(), String> {
            if Some(self.env().caller()) != self.owner {
                return Err("Only owner can reactivate devices".into());
            }

            let account_bytes = ink_account_to_bytes(account);
            let mut device = self.devices.get(account_bytes)
                .ok_or("Device not registered")?;
            
            device.active = true;
            self.devices.insert(account_bytes, &device);

            Ok(())
        }

        /// Get all authorized callers (owner only)
        #[ink(message)]
        pub fn get_authorized_callers(&self) -> Result<Vec<AccountId>, String> {
            if Some(self.env().caller()) != self.owner {
                return Err("Only owner can view authorized callers".into());
            }
            // Not iterable safely; return empty list to avoid O(n)
            Ok(Vec::new())
        }

        /// Check if an account is authorized
        #[ink(message)]
        pub fn is_authorized_caller(&self, account: AccountId) -> bool {
            if Some(account) == self.owner { return true; }
            self.authorized_callers.get(account).unwrap_or(false)
        }

        /// Check if caller is authorized
        fn ensure_authorized(&self) -> Result<(), String> {
            let caller = self.env().caller();
            if Some(caller) == self.owner || self.authorized_callers.get(caller).unwrap_or(false) {
                Ok(())
            } else {
                Err("Unauthorized caller".into())
            }
        }

        /// Withdraw stake (down to minimum if active)
        #[ink(message)]
        pub fn withdraw_stake(&mut self, amount: Balance) -> Result<(), String> {
            if self.entered { return Err("Reentrancy".into()); }
            self.entered = true;
            if self.paused { self.entered = false; return Err("Paused".into()); }
            let caller = self.env().caller();
            let caller_bytes = ink_account_to_bytes(caller);
            let mut device = self.devices.get(caller_bytes).ok_or("Device not registered")?;
            if amount == 0 { return Ok(()); }
            if amount > device.stake { return Err("AmountExceedsStake".into()); }
            let remaining = device.stake.saturating_sub(amount);
            if device.active && remaining < self.min_stake { return Err("BelowMinStake".into()); }
            device.stake = remaining;
            self.devices.insert(caller_bytes, &device);
            self.env().transfer(caller, amount).map_err(|_| String::from("TransferFailed"))?;
            self.env().emit_event(StakeWithdrawn { account: caller, amount, remaining_stake: remaining });
            self.entered = false;
            Ok(())
        }

        /// Slash stake (owner/governance)
        #[ink(message)]
        pub fn slash_stake(&mut self, account: AccountId, amount: Balance, reason: String) -> Result<(), String> {
            if self.entered { return Err("Reentrancy".into()); }
            self.entered = true;
            let sender = self.env().caller();
            if Some(sender) != self.owner && Some(sender) != self.governance_address { return Err("Unauthorized".into()); }
            let acc_bytes = ink_account_to_bytes(account);
            let mut device = self.devices.get(acc_bytes).ok_or("Device not registered")?;
            let slash_amt = core::cmp::min(amount, device.stake);
            device.stake = device.stake.saturating_sub(slash_amt);
            if device.stake < self.min_stake { device.active = false; }
            self.devices.insert(acc_bytes, &device);
            self.env().emit_event(StakeSlashed { account, amount: slash_amt, remaining_stake: device.stake, reason });
            self.entered = false;
            Ok(())
        }

        /// Pause/unpause (owner or governance)
        #[ink(message)]
        pub fn set_paused(&mut self, pause: bool) -> Result<(), String> {
            let sender = self.env().caller();
            if Some(sender) != self.owner && Some(sender) != self.governance_address { return Err("Unauthorized".into()); }
            self.paused = pause;
            Ok(())
        }

        /// Get device stake
        #[ink(message)]
        pub fn get_device_stake(&self, account: AccountId) -> Option<Balance> {
            let account_bytes = ink_account_to_bytes(account);
            self.devices.get(account_bytes).map(|device| device.stake)
        }

        /// Get device activity status
        #[ink(message)]
        pub fn is_device_active(&self, account: AccountId) -> Option<bool> {
            let account_bytes = ink_account_to_bytes(account);
            self.devices.get(account_bytes).map(|device| device.active)
        }

        /// Get device energy contribution
        #[ink(message)]
        pub fn get_device_energy_contribution(&self, account: AccountId) -> Option<u64> {
            let account_bytes = ink_account_to_bytes(account);
            self.devices.get(account_bytes).map(|device| device.total_energy_contributed)
        }

        /// Get device event statistics
        #[ink(message)]
        pub fn get_device_event_stats(&self, account: AccountId) -> Option<(u32, u32)> {
            let account_bytes = ink_account_to_bytes(account);
            self.devices.get(account_bytes).map(|device| (device.successful_events, device.failed_events))
        }
    }
    

    #[cfg(test)]
    mod tests {
        use super::*;
        use powergrid_shared::DeviceType;
        use ink::env::test::{default_accounts, set_caller, set_value_transferred, DefaultAccounts};
        use ink::env::DefaultEnvironment;

        #[ink::test]
        fn test_device_registration_success() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            let mut registry = ResourceRegistry::new(1000);

            set_caller::<DefaultEnvironment>(accounts.alice);
            set_value_transferred::<DefaultEnvironment>(1500);

            let metadata = DeviceMetadata {
                device_type: DeviceType::SmartPlug,
                capacity_watts: 2000,
                location: "Home".into(),
                manufacturer: "Tesla".into(),
                model: "Model S".into(),
                firmware_version: "1.0.0".into(),
                installation_date: 1640995200,
            };

            let result = registry.register_device(metadata);
            assert!(result.is_ok());
            assert_eq!(registry.get_device_count(), 1);
            assert!(registry.is_device_registered(accounts.alice));
        }

        #[ink::test]
        fn test_device_registration_insufficient_stake() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            let mut registry = ResourceRegistry::new(1000);

            set_caller::<DefaultEnvironment>(accounts.alice);
            set_value_transferred::<DefaultEnvironment>(500); // Below minimum

            let metadata = DeviceMetadata {
                device_type: DeviceType::SmartPlug,
                capacity_watts: 2000,
                location: "Home".into(),
                manufacturer: "Tesla".into(),
                model: "Model S".into(),
                firmware_version: "1.0.0".into(),
                installation_date: 1640995200,
            };

            let result = registry.register_device(metadata);
            assert!(result.is_err());
            assert_eq!(registry.get_device_count(), 0);
        }

        #[ink::test]
        fn test_device_deactivation() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            let mut registry = ResourceRegistry::new(1000);

            // Register device first
            set_caller::<DefaultEnvironment>(accounts.alice);
            set_value_transferred::<DefaultEnvironment>(1500);

            let metadata = DeviceMetadata {
                device_type: DeviceType::SmartPlug,
                capacity_watts: 2000,
                location: "Home".into(),
                manufacturer: "Tesla".into(),
                model: "Model S".into(),
                firmware_version: "1.0.0".into(),
                installation_date: 1640995200,
            };

            let _ = registry.register_device(metadata);

            // Deactivate as owner
            let result = registry.deactivate_device(accounts.alice, "Test deactivation".into());
            assert!(result.is_ok());

            let device = registry.get_device(accounts.alice).unwrap();
            assert!(!device.active);
            assert_eq!(registry.is_device_active(accounts.alice), Some(false));
        }

        #[ink::test]
        fn test_stake_increase() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            let mut registry = ResourceRegistry::new(1000);

            // Register device first
            set_caller::<DefaultEnvironment>(accounts.alice);
            set_value_transferred::<DefaultEnvironment>(1500);

            let metadata = DeviceMetadata {
                device_type: DeviceType::SmartPlug,
                capacity_watts: 2000,
                location: "Home".into(),
                manufacturer: "Tesla".into(),
                model: "Model S".into(),
                firmware_version: "1.0.0".into(),
                installation_date: 1640995200,
            };

            let _ = registry.register_device(metadata);

            // Increase stake
            set_value_transferred::<DefaultEnvironment>(500);
            let result = registry.increase_stake();
            assert!(result.is_ok());

            assert_eq!(registry.get_device_stake(accounts.alice), Some(2000));
        }

        #[ink::test]
        fn test_authorized_caller_management() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            let mut registry = ResourceRegistry::new(1000);

            // Add authorized caller
            let result = registry.add_authorized_caller(accounts.bob);
            assert!(result.is_ok());
            assert!(registry.is_authorized_caller(accounts.bob));

            // Remove authorized caller
            let result = registry.remove_authorized_caller(accounts.bob);
            assert!(result.is_ok());
            assert!(!registry.is_authorized_caller(accounts.bob));
        }
    }
}