use ink::prelude::string::String;

/// Interface for token operations
pub trait TokenInterface {
    fn transfer(&mut self, to: [u8; 32], value: u128) -> bool;
    fn balance_of(&self, owner: [u8; 32]) -> u128;
    fn mint(&mut self, to: [u8; 32], value: u128) -> bool;
    fn total_supply(&self) -> u128;
    fn approve(&mut self, spender: [u8; 32], value: u128) -> bool;
    fn allowance(&self, owner: [u8; 32], spender: [u8; 32]) -> u128;
    fn transfer_from(&mut self, from: [u8; 32], to: [u8; 32], value: u128) -> bool;
}

/// Interface for device registration and management
pub trait RegistryInterface {
    fn is_device_registered(&self, account: [u8; 32]) -> bool;
    fn get_device_reputation(&self, account: [u8; 32]) -> Option<u32>;
    fn update_device_performance(&mut self, account: [u8; 32], energy_contributed: u64, success: bool);
}

/// Additional device-specific interface
pub trait DeviceRegistryInterface {
    fn get_device_count(&self) -> u64;
    fn get_min_stake(&self) -> u128;
}

/// Interface for grid service operations
pub trait GridServiceInterface {
    fn create_grid_event(&mut self, event_type: crate::GridEventType, duration_minutes: u64, 
                        compensation_rate: u128, target_reduction_kw: u64) -> Result<u64, String>;
    fn participate_in_event(&mut self, event_id: u64, energy_reduction_wh: u64) -> Result<(), String>;
    fn verify_participation(&mut self, event_id: u64, participant: [u8; 32], 
                           actual_reduction: u64) -> Result<(), String>;
}

/// Interface for governance operations  
pub trait GovernanceInterface {
    fn create_proposal(&mut self, proposal_type: crate::ProposalType, description: String) -> Result<u64, String>;
    fn vote(&mut self, proposal_id: u64, support: bool, reason: String) -> Result<(), String>;
    fn execute_proposal(&mut self, proposal_id: u64) -> Result<(), String>;
}
