#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod resource_registry {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;
use powergrid_shared::{DeviceMetadata, Device, ink_account_to_bytes};

    /// The ResourceRegistry contract
    #[ink(storage)]
    pub struct ResourceRegistry {
        /// Mapping from AccountId to Device info (using [u8; 32] as key)
        devices: Mapping<[u8; 32], Device>,
        /// Minimum stake required for device registration
        min_stake: Balance,
        /// Owner of the contract (using ink! AccountId for env() compatibility)
        owner: AccountId,
        /// Total number of registered devices
        device_count: u64,
        /// Authorized callers (using ink! AccountId for env() compatibility)
        authorized_callers: Vec<AccountId>,
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

    impl ResourceRegistry {
        /// Constructor
        #[ink(constructor)]
        pub fn new(min_stake: Balance) -> Self {
            Self {
                devices: Mapping::default(),
                min_stake,
                owner: Self::env().caller(),
                device_count: 0,
                authorized_callers: Vec::new(),
            }
        }

        /// Register a new device with stake
        #[ink(message, payable)]
        pub fn register_device(&mut self, metadata: DeviceMetadata) -> Result<(), String> {
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
            };

            self.devices.insert(caller_bytes, &device);
            self.device_count = self.device_count.saturating_add(1);

            self.env().emit_event(DeviceRegistered {
                account: caller,
                stake,
                reputation: device.reputation,
            });

            Ok(())
        }

        /// Increase stake for existing device
        #[ink(message, payable)]
        pub fn increase_stake(&mut self) -> Result<(), String> {
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
            
            self.devices.insert(account_bytes, &device);

            self.env().emit_event(ReputationUpdated {
                account,
                old_reputation,
                new_reputation: device.reputation,
            });

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
            if self.env().caller() != self.owner {
                return Err("Only owner can update minimum stake".into());
            }
            
            self.min_stake = new_min_stake;
            Ok(())
        }

        /// Add authorized caller (owner only)
        #[ink(message)]
        pub fn add_authorized_caller(&mut self, caller: AccountId) -> Result<(), String> {
            if self.env().caller() != self.owner {
                return Err("Only owner can add authorized callers".into());
            }
            
            self.authorized_callers.push(caller);
            Ok(())
        }

        /// Remove authorized caller (owner only)
        #[ink(message)]
        pub fn remove_authorized_caller(&mut self, caller: AccountId) -> Result<(), String> {
            if self.env().caller() != self.owner {
                return Err("Only owner can remove authorized callers".into());
            }
            
            self.authorized_callers.retain(|&x| x != caller);
            Ok(())
        }

        /// Deactivate a device (owner only)
        #[ink(message)]
        pub fn deactivate_device(&mut self, account: AccountId, reason: String) -> Result<(), String> {
            if self.env().caller() != self.owner {
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
            if self.env().caller() != self.owner {
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
            if self.env().caller() != self.owner {
                return Err("Only owner can view authorized callers".into());
            }
            
            Ok(self.authorized_callers.clone())
        }

        /// Check if caller is authorized
        fn ensure_authorized(&self) -> Result<(), String> {
            let caller = self.env().caller();
            if caller == self.owner || self.authorized_callers.contains(&caller) {
                Ok(())
            } else {
                Err("Unauthorized caller".into())
            }
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

            // Check authorized callers
            let callers = registry.get_authorized_callers().unwrap();
            assert!(callers.contains(&accounts.bob));

            // Remove authorized caller
            let result = registry.remove_authorized_caller(accounts.bob);
            assert!(result.is_ok());

            let callers = registry.get_authorized_callers().unwrap();
            assert!(!callers.contains(&accounts.bob));
        }
    }
}