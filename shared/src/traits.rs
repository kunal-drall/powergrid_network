use crate::{AccountId, Balance};
use ink::prelude::vec::Vec;

#[ink::trait_definition]
pub trait TokenInterface {
    #[ink(message)]
    fn balance_of(&self, owner: AccountId) -> Balance;
    
    #[ink(message)]
    fn transfer(&mut self, to: AccountId, value: Balance) -> bool;
    
    #[ink(message)]
    fn mint(&mut self, to: AccountId, value: Balance) -> bool;
    
    #[ink(message)]
    fn burn(&mut self, from: AccountId, value: Balance) -> bool;
    
    #[ink(message)]
    fn total_supply(&self) -> Balance;
}

#[ink::trait_definition]
pub trait ResourceRegistryInterface {
    #[ink(message)]
    fn is_device_registered(&self, account: AccountId) -> bool;
    
    #[ink(message)]
    fn get_device_reputation(&self, account: AccountId) -> Option<u32>;
    
    #[ink(message)]
    fn update_device_performance(&mut self, account: AccountId, energy_contributed: u64, success: bool);
}

#[ink::trait_definition]
pub trait GridServiceInterface {
    #[ink(message)]
    fn calculate_event_rewards(&self, event_id: u64) -> Vec<(AccountId, Balance)>;
    
    #[ink(message)]
    fn verify_participation(&mut self, event_id: u64, participant: AccountId, energy_contributed: u64);
}