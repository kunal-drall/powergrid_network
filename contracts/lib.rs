#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::DefaultEnvironment;
use ink::prelude::string::String;
use ink::prelude::collections::BTreeMap;
use ink::storage::traits::{Packed, StorageLayout};
use ink::storage::Mapping;
use scale::{Decode, Encode};
use scale_info::TypeInfo;

type AccountId = <DefaultEnvironment as ink::env::Environment>::AccountId;
type Balance = <DefaultEnvironment as ink::env::Environment>::Balance;

#[derive(Decode, Encode, Clone, TypeInfo, StorageLayout)]
pub struct Device {
    device_type: String,
    capacity: u64,
    location: String,
    stake: Balance,
    reputation: u32,
}

#[ink::contract]
mod resource_registry {
    use super::*;

    #[ink(storage)]
    #[derive(StorageLayout)]
    pub struct ResourceRegistry {
        min_stake: Balance,
        devices: Mapping<AccountId, Device>,
        reputations: Mapping<AccountId, u32>,
    }

    impl ResourceRegistry {
        #[ink(constructor)]
        pub fn new(min_stake: Balance) -> Self {
            Self {
                min_stake,
                devices: Mapping::default(),
                reputations: Mapping::default(),
            }
        }

        #[ink(message, payable)]
        pub fn register_device(&mut self, device_type: String, capacity: u64, location: String) {
            let caller = self.env().caller();
            let stake = self.env().transferred_value();
            assert!(stake >= self.min_stake, "Insufficient stake");
            let device = Device {
                device_type,
                capacity,
                location,
                stake,
                reputation: 100,
            };
            self.devices.insert(caller, &device);
            self.reputations.insert(caller, &100);
        }

        #[ink(message)]
        pub fn get_device(&self, account: AccountId) -> Option<Device> {
            self.devices.get(account)
        }

        #[ink(message)]
        pub fn update_reputation(&mut self, account: AccountId, delta: i32) {
            let mut rep = self.reputations.get(account).unwrap_or(100);
            rep = (rep as i32 + delta).max(0) as u32;
            self.reputations.insert(account, &rep);
        }
    }
}

#[derive(Decode, Encode, Clone, TypeInfo, StorageLayout)]
pub struct GridEvent {
    event_type: String,
    duration: u64,
    compensation_rate: Balance,
    active: bool,
}

#[ink::contract]
mod grid_service {
    use super::*;

    #[ink(storage)]
    #[derive(StorageLayout)]
    pub struct GridService {
        token_address: AccountId,
        events: Mapping<u64, GridEvent>,
        event_count: u64,
        participation: BTreeMap<(u64, AccountId), Balance>,
    }

    impl GridService {
        #[ink(constructor)]
        pub fn new(token_address: AccountId) -> Self {
            Self {
                token_address,
                events: Mapping::default(),
                event_count: 0,
                participation: BTreeMap::new(),
            }
        }

        #[ink(message)]
        pub fn create_event(&mut self, event_type: String, duration: u64, compensation_rate: Balance) {
            let event_id = self.event_count;
            let event = GridEvent {
                event_type,
                duration,
                compensation_rate,
                active: true,
            };
            self.events.insert(event_id, &event);
            self.event_count += 1;
        }

        #[ink(message)]
        pub fn participate(&mut self, event_id: u64) {
            let caller = self.env().caller();
            let event = self.events.get(event_id).expect("Event not found");
            assert!(event.active, "Event not active");
            self.participation.insert((event_id, caller), 1);
        }

        #[ink(message)]
        pub fn end_event(&mut self, event_id: u64) {
            let mut event = self.events.get(event_id).expect("Event not found");
            event.active = false;
            self.events.insert(event_id, &event);
        }
    }
}

#[ink::contract]
mod token {
    use super::*;

    #[ink(storage)]
    #[derive(StorageLayout)]
    pub struct Token {
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: AccountId,
        value: Balance,
    }

    impl Token {
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut balances = Mapping::default();
            balances.insert(Self::env().caller(), &total_supply);
            Self::env().emit_event(Transfer {
                from: None,
                to: Self::env().caller(),
                value: total_supply,
            });
            Self {
                total_supply,
                balances,
            }
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap_or(0)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) {
            let from = self.env().caller();
            let from_balance = self.balance_of(from);
            assert!(from_balance >= value, "Insufficient balance");
            self.balances.insert(from, &(from_balance - value));
            let to_balance = self.balance_of(to);
            self.balances.insert(to, &(to_balance + value));
            self.env().emit_event(Transfer {
                from: Some(from),
                to,
                value,
            });
        }

        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, value: Balance) {
            let caller = self.env().caller();
            // Add access control if needed
            self.total_supply += value;
            let balance = self.balance_of(to);
            self.balances.insert(to, &(balance + value));
            self.env().emit_event(Transfer {
                from: None,
                to,
                value,
            });
        }
    }
}

#[derive(Decode, Encode, Clone, TypeInfo, StorageLayout)]
pub struct Proposal {
    description: String,
    yes_votes: u64,
    no_votes: u64,
    active: bool,
}

#[ink::contract]
mod governance {
    use super::*;

    #[ink(storage)]
    #[derive(StorageLayout)]
    pub struct Governance {
        token_address: AccountId,
        proposals: Mapping<u64, Proposal>,
        proposal_count: u64,
        votes: BTreeMap<(u64, AccountId), bool>,
    }

    impl Governance {
        #[ink(constructor)]
        pub fn new(token_address: AccountId) -> Self {
            Self {
                token_address,
                proposals: Mapping::default(),
                proposal_count: 0,
                votes: BTreeMap::new(),
            }
        }

        #[ink(message)]
        pub fn create_proposal(&mut self, description: String) {
            let proposal_id = self.proposal_count;
            let proposal = Proposal {
                description,
                yes_votes: 0,
                no_votes: 0,
                active: true,
            };
            self.proposals.insert(proposal_id, &proposal);
            self.proposal_count += 1;
        }

        #[ink(message)]
        pub fn vote(&mut self, proposal_id: u64, vote: bool) {
            let caller = self.env().caller();
            let mut proposal = self.proposals.get(proposal_id).expect("Proposal not found");
            assert!(proposal.active, "Proposal not active");
            assert!(self.votes.get(&(proposal_id, caller)).is_none(), "Already voted");
            if vote {
                proposal.yes_votes += 1;
            } else {
                proposal.no_votes += 1;
            }
            self.votes.insert((proposal_id, caller), vote);
            self.proposals.insert(proposal_id, &proposal);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ink::env::test::{default_accounts, set_caller, set_value_transferred};

    #[ink::test]
    fn register_device_works() {
        let accounts = default_accounts::<DefaultEnvironment>();
        let mut registry = resource_registry::ResourceRegistry::new(100);
        set_caller::<DefaultEnvironment>(accounts.alice);
        set_value_transferred::<DefaultEnvironment>(100);
        registry.register_device("SmartPlug".into(), 1000, "Delhi".into());
        let device = registry.get_device(accounts.alice).unwrap();
        assert_eq!(device.capacity, 1000);
    }

    #[ink::test]
    fn grid_event_works() {
        let accounts = default_accounts::<DefaultEnvironment>();
        let mut grid = grid_service::GridService::new(accounts.bob);
        grid.create_event("DemandResponse".into(), 60, 10);
        set_caller::<DefaultEnvironment>(accounts.alice);
        grid.participate(0);
        assert!(grid.participation.get(&(0, accounts.alice)).is_some());
    }

    #[ink::test]
    fn token_transfer_works() {
        let accounts = default_accounts::<DefaultEnvironment>();
        let mut token = token::Token::new(1000000);
        set_caller::<DefaultEnvironment>(accounts.alice);
        token.transfer(accounts.bob, 100);
        assert_eq!(token.balance_of(accounts.bob), 100);
    }

    #[ink::test]
    fn governance_vote_works() {
        let accounts = default_accounts::<DefaultEnvironment>();
        let mut governance = governance::Governance::new(accounts.bob);
        governance.create_proposal("Update min_stake".into());
        set_caller::<DefaultEnvironment>(accounts.alice);
        governance.vote(0, true);
        let proposal = governance.proposals.get(&0).unwrap();
        assert_eq!(proposal.yes_votes, 1);
    }
}