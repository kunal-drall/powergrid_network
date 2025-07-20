#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod resource_registry {
    use powergrid_shared::{
    Device, DeviceMetadata, DeviceType, ResourceRegistryInterface,
};
    use ink::storage::Mapping;
    use ink::prelude::string::String;

    #[ink(storage)]
    pub struct ResourceRegistry {
        // Configuration
        min_stake: Balance,
        admin: AccountId,
        governance_address: Option<AccountId>,
        
        // Core storage
        devices: Mapping<AccountId, Device>,
        device_count: u32,
        
        // Performance tracking
        reputation_scores: Mapping<AccountId, u32>,
        performance_history: Mapping<(AccountId, u64), u64>, // (account, timestamp) -> energy_contributed
        
        // Access control
        authorized_contracts: Mapping<AccountId, bool>,
    }

    #[ink(event)]
    pub struct DeviceRegistered {
        #[ink(topic)]
        owner: AccountId,
        device_type: DeviceType,
        capacity: u64,
        stake: Balance,
    }

    #[ink(event)]
    pub struct ReputationUpdated {
        #[ink(topic)]
        owner: AccountId,
        old_reputation: u32,
        new_reputation: u32,
        reason: String,
    }

    #[ink(event)]
    pub struct DeviceDeactivated {
        #[ink(topic)]
        owner: AccountId,
        reason: String,
    }

    impl ResourceRegistry {
        #[ink(constructor)]
        pub fn new(min_stake: Balance) -> Self {
            Self {
                min_stake,
                admin: Self::env().caller(),
                governance_address: None,
                devices: Mapping::default(),
                device_count: 0,
                reputation_scores: Mapping::default(),
                performance_history: Mapping::default(),
                authorized_contracts: Mapping::default(),
            }
        }

        #[ink(message, payable)]
        pub fn register_device(&mut self, metadata: DeviceMetadata) -> Result<(), String> {
            let caller = self.env().caller();
            let stake = self.env().transferred_value();
            let current_time = self.env().block_timestamp();

            // Validate stake amount
            if stake < self.min_stake {
                return Err("Insufficient stake amount".into());
            }

            // Check if device already registered
            if self.devices.contains(caller) {
                return Err("Device already registered".into());
            }

            // Validate metadata
            self.validate_device_metadata(&metadata)?;

            let device = Device {
                metadata: metadata.clone(),
                stake,
                reputation: 100, // Starting reputation
                total_energy_contributed: 0,
                successful_events: 0,
                failed_events: 0,
                last_activity: current_time,
                active: true,
            };

            self.devices.insert(caller, &device);
            self.reputation_scores.insert(caller, &100);
            self.device_count += 1;

            self.env().emit_event(DeviceRegistered {
                owner: caller,
                device_type: metadata.device_type,
                capacity: metadata.capacity_watts,
                stake,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn get_device(&self, account: AccountId) -> Option<Device> {
            self.devices.get(account)
        }

        #[ink(message)]
        pub fn update_device_metadata(&mut self, new_metadata: DeviceMetadata) -> Result<(), String> {
            let caller = self.env().caller();
            
            let mut device = self.devices.get(caller)
                .ok_or("Device not registered")?;

            self.validate_device_metadata(&new_metadata)?;
            
            device.metadata = new_metadata;
            device.last_activity = self.env().block_timestamp();
            
            self.devices.insert(caller, &device);
            Ok(())
        }

        #[ink(message)]
        pub fn deactivate_device(&mut self) -> Result<(), String> {
            let caller = self.env().caller();
            
            let mut device = self.devices.get(caller)
                .ok_or("Device not registered")?;

            if !device.active {
                return Err("Device already deactivated".into());
            }

            device.active = false;
            self.devices.insert(caller, &device);

            self.env().emit_event(DeviceDeactivated {
                owner: caller,
                reason: "User requested deactivation".into(),
            });

            Ok(())
        }

        #[ink(message)]
        pub fn increase_stake(&mut self) -> Result<(), String> {
            let caller = self.env().caller();
            let additional_stake = self.env().transferred_value();
            
            let mut device = self.devices.get(caller)
                .ok_or("Device not registered")?;

            device.stake += additional_stake;
            self.devices.insert(caller, &device);
            
            Ok(())
        }

        // Performance scoring algorithm
        fn calculate_performance_score(&self, device: &Device) -> u32 {
            if device.successful_events + device.failed_events == 0 {
                return 100; // Default score for new devices
            }

            let success_rate = (device.successful_events * 100) / 
                (device.successful_events + device.failed_events);
            
            let energy_factor = (device.total_energy_contributed / 1000).min(50); // Up to 50 points for energy
            
            let base_score = success_rate + energy_factor as u32;
            base_score.min(100).max(1) // Keep between 1-100
        }

        fn validate_device_metadata(&self, metadata: &DeviceMetadata) -> Result<(), String> {
            if metadata.capacity_watts == 0 {
                return Err("Device capacity must be greater than 0".into());
            }

            if metadata.location.is_empty() {
                return Err("Device location cannot be empty".into());
            }

            if metadata.manufacturer.is_empty() {
                return Err("Manufacturer cannot be empty".into());
            }

            Ok(())
        }

        // ========================================================================
        // CROSS-CONTRACT INTERFACE IMPLEMENTATION
        // ========================================================================

        #[ink(message)]
        pub fn authorize_contract(&mut self, contract_address: AccountId) -> Result<(), String> {
            self.ensure_admin_or_governance()?;
            self.authorized_contracts.insert(contract_address, &true);
            Ok(())
        }

        #[ink(message)]
        pub fn set_governance_address(&mut self, governance_address: AccountId) -> Result<(), String> {
            if self.env().caller() != self.admin {
                return Err("Only admin can set governance address".into());
            }
            self.governance_address = Some(governance_address);
            Ok(())
        }

        fn ensure_authorized(&self) -> Result<(), String> {
            let caller = self.env().caller();
            if caller == self.admin || 
               self.governance_address == Some(caller) ||
               self.authorized_contracts.get(caller).unwrap_or(false) {
                Ok(())
            } else {
                Err("Unauthorized access".into())
            }
        }

        fn ensure_admin_or_governance(&self) -> Result<(), String> {
            let caller = self.env().caller();
            if caller == self.admin || self.governance_address == Some(caller) {
                Ok(())
            } else {
                Err("Admin or governance access required".into())
            }
        }

        #[ink(message)]
        pub fn update_min_stake(&mut self, new_min_stake: Balance) -> Result<(), String> {
            self.ensure_admin_or_governance()?;
            self.min_stake = new_min_stake;
            Ok(())
        }

        #[ink(message)]
        pub fn get_min_stake(&self) -> Balance {
            self.min_stake
        }

        #[ink(message)]
        pub fn get_device_count(&self) -> u32 {
            self.device_count
        }

        #[ink(message)]
        pub fn get_active_device_count(&self) -> u32 {
            // In a real implementation, this would iterate through devices
            // For now, return total count (simplified)
            self.device_count
        }
    }

    impl ResourceRegistryInterface for ResourceRegistry {
        #[ink(message)]
        fn is_device_registered(&self, account: AccountId) -> bool {
            self.devices.get(account).map(|d| d.active).unwrap_or(false)
        }

        #[ink(message)]
        fn get_device_reputation(&self, account: AccountId) -> Option<u32> {
            self.reputation_scores.get(account)
        }

        #[ink(message)]
        fn update_device_performance(&mut self, account: AccountId, energy_contributed: u64, success: bool) {
            if let Err(_) = self.ensure_authorized() {
                return; // Silently fail unauthorized calls
            }

            if let Some(mut device) = self.devices.get(account) {
                let old_reputation = device.reputation;
                
                device.total_energy_contributed += energy_contributed;
                if success {
                    device.successful_events += 1;
                } else {
                    device.failed_events += 1;
                }
                device.last_activity = self.env().block_timestamp();
                
                let new_reputation = self.calculate_performance_score(&device);
                device.reputation = new_reputation;
                
                self.devices.insert(account, &device);
                self.reputation_scores.insert(account, &new_reputation);
                
                // Record performance history
                let timestamp = self.env().block_timestamp();
                self.performance_history.insert((account, timestamp), &energy_contributed);

                self.env().emit_event(ReputationUpdated {
                    owner: account,
                    old_reputation,
                    new_reputation,
                    reason: if success { "Successful event participation".into() } else { "Failed event participation".into() },
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ink::env::test::{default_accounts, set_caller, set_value_transferred};
    use powergrid_shared::{DeviceMetadata, DeviceType};

    fn create_sample_device_metadata() -> DeviceMetadata {
        DeviceMetadata {
            device_type: DeviceType::SmartPlug,
            capacity_watts: 2000,
            location: "Delhi, India".into(),
            manufacturer: "SmartCorp".into(),
            model: "SP-2000".into(),
            firmware_version: "1.0.0".into(),
            installation_date: 1640995200000,
        }
    }

    #[ink::test]
    fn test_device_registration_success() {
        let accounts = default_accounts();
        let mut registry = resource_registry::ResourceRegistry::new(1000);

        set_caller(accounts.alice);
        set_value_transferred(1500);

        let metadata = create_sample_device_metadata();
        let result = registry.register_device(metadata.clone());

        assert!(result.is_ok());

        let device = registry.get_device(accounts.alice).unwrap();
        assert_eq!(device.metadata.device_type, DeviceType::SmartPlug);
        assert_eq!(device.metadata.capacity_watts, 2000);
        assert_eq!(device.stake, 1500);
        assert_eq!(device.reputation, 100);
        assert!(device.active);
    }

    #[ink::test]
    fn test_device_registration_insufficient_stake() {
        let accounts = default_accounts();
        let mut registry = resource_registry::ResourceRegistry::new(1000);

        set_caller(accounts.alice);
        set_value_transferred(500); // Below min stake

        let metadata = create_sample_device_metadata();
        let result = registry.register_device(metadata);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient stake amount");
    }

    #[ink::test]
    fn test_device_deactivation() {
        let accounts = default_accounts();
        let mut registry = resource_registry::ResourceRegistry::new(1000);

        set_caller(accounts.alice);
        set_value_transferred(1500);

        let metadata = create_sample_device_metadata();
        assert!(registry.register_device(metadata).is_ok());

        // Deactivate device
        let result = registry.deactivate_device();
        assert!(result.is_ok());

        let device = registry.get_device(accounts.alice).unwrap();
        assert!(!device.active);
    }
}