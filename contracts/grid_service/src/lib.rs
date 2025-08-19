#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod grid_service {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;
use powergrid_shared::{GridEvent, GridEventType, Participation, ink_account_to_bytes};

    /// The GridService contract
    #[ink(storage)]
    pub struct GridService {
        /// Contract owner
        owner: AccountId,
        /// Token contract address for rewards
        token_address: AccountId,
        /// Registry contract address for device verification
        registry_address: AccountId,
        /// Grid events mapping
        events: Mapping<u64, GridEvent>,
        /// Event participations mapping (event_id -> Vec<Participation>)
        participations: Mapping<u64, Vec<Participation>>,
        /// Next event ID
        next_event_id: u64,
        /// Authorized callers
        authorized_callers: Vec<AccountId>,
    }

    /// Events emitted by the contract
    #[ink(event)]
    pub struct GridEventCreated {
        #[ink(topic)]
        event_id: u64,
        event_type: GridEventType,
        compensation_rate: Balance,
        target_reduction_kw: u64,
        start_time: u64,
        end_time: u64,
    }

    #[ink(event)]
    pub struct ParticipationRecorded {
        #[ink(topic)]
        event_id: u64,
        #[ink(topic)]
        participant: AccountId,
        energy_contributed_wh: u64,
    }

    #[ink(event)]
    pub struct ParticipationVerified {
        #[ink(topic)]
        event_id: u64,
        #[ink(topic)]
        participant: AccountId,
        reward_earned: Balance,
        verified: bool,
    }

    #[ink(event)]
    pub struct GridEventCompleted {
        #[ink(topic)]
        event_id: u64,
        total_participants: u32,
        total_energy_reduced: u64,
    }

    impl GridService {
        /// Constructor
        #[ink(constructor)]
        pub fn new(token_address: AccountId, registry_address: AccountId) -> Self {
            Self {
                owner: Self::env().caller(),
                token_address,
                registry_address,
                events: Mapping::default(),
                participations: Mapping::default(),
                next_event_id: 1,
                authorized_callers: Vec::new(),
            }
        }

        /// Create a new grid event
        #[ink(message)]
        pub fn create_grid_event(
            &mut self,
            event_type: GridEventType,
            duration_minutes: u64,
            compensation_rate: Balance,
            target_reduction_kw: u64,
        ) -> Result<u64, String> {
            if self.ensure_authorized().is_err() {
                return Err("Unauthorized caller".into());
            }

            let now = self.env().block_timestamp();
            let event_id = self.next_event_id;
            
            let event = GridEvent {
                event_type: event_type.clone(),
                duration_minutes,
                base_compensation_rate: compensation_rate,
                target_reduction_kw,
                created_at: now,
                start_time: now,
                end_time: now.saturating_add(duration_minutes.saturating_mul(60_000)), // Convert to milliseconds
                active: true,
                total_participants: 0,
                total_energy_reduced: 0,
                completed: false,
            };

            self.events.insert(event_id, &event);
            self.next_event_id = self.next_event_id.saturating_add(1);

            self.env().emit_event(GridEventCreated {
                event_id,
                event_type,
                compensation_rate,
                target_reduction_kw,
                start_time: event.start_time,
                end_time: event.end_time,
            });

            Ok(event_id)
        }

        /// Participate in a grid event
        #[ink(message)]
        pub fn participate_in_event(&mut self, event_id: u64, energy_reduction_wh: u64) -> Result<(), String> {
            let caller = self.env().caller();
            let caller_bytes = ink_account_to_bytes(caller);
            
            // Verify event exists and is active
            let mut event = self.events.get(event_id)
                .ok_or("Event not found")?;
            
            if !event.active {
                return Err("Event is not active".into());
            }

            let now = self.env().block_timestamp();
            if now > event.end_time {
                return Err("Event has ended".into());
            }

            // Create participation record
            let participation = Participation {
                participant: caller_bytes,
                energy_contributed_wh: energy_reduction_wh,
                participation_start: now,
                participation_end: 0, // Will be set when verified
                reward_earned: 0,    // Will be calculated when verified
                verified: false,
            };

            // Add to participations
            let mut participations = self.participations.get(event_id).unwrap_or_default();
            participations.push(participation);
            self.participations.insert(event_id, &participations);

            // Update event stats
            event.total_participants = event.total_participants.saturating_add(1);
            event.total_energy_reduced = event.total_energy_reduced.saturating_add(energy_reduction_wh);
            self.events.insert(event_id, &event);

            self.env().emit_event(ParticipationRecorded {
                event_id,
                participant: caller,
                energy_contributed_wh: energy_reduction_wh,
            });

            Ok(())
        }

        /// Verify participation and distribute rewards (authorized only)
        #[ink(message)]
        pub fn verify_participation(
            &mut self,
            event_id: u64,
            participant: AccountId,
            actual_reduction: u64,
        ) -> Result<(), String> {
            if self.ensure_authorized().is_err() {
                return Err("Unauthorized caller".into());
            }

            let participant_bytes = ink_account_to_bytes(participant);
            let mut participations = self.participations.get(event_id)
                .ok_or("No participations found for event")?;

            let event = self.events.get(event_id)
                .ok_or("Event not found")?;

            // Find and update the participation
            let mut found = false;
            for participation in participations.iter_mut() {
                if participation.participant == participant_bytes {
                    participation.energy_contributed_wh = actual_reduction;
                    participation.participation_end = self.env().block_timestamp();
                    participation.verified = true;
                    
                    // Calculate reward (simplified calculation)
                    participation.reward_earned = self.calculate_reward(&event, actual_reduction);
                    
                    found = true;
                    break;
                }
            }

            if !found {
                return Err("Participation not found".into());
            }

            self.participations.insert(event_id, &participations);

            // Find the updated participation for the reward amount
            let reward_earned = participations.iter()
                .find(|p| p.participant == participant_bytes)
                .map(|p| p.reward_earned)
                .unwrap_or(0);

            self.env().emit_event(ParticipationVerified {
                event_id,
                participant,
                reward_earned,
                verified: true,
            });

            Ok(())
        }

        /// Get grid event details
        #[ink(message)]
        pub fn get_grid_event(&self, event_id: u64) -> Option<GridEvent> {
            self.events.get(event_id)
        }

        /// Get event participations
        #[ink(message)]
        pub fn get_event_participations(&self, event_id: u64) -> Vec<Participation> {
            self.participations.get(event_id).unwrap_or_default()
        }

        /// Complete a grid event (authorized only)
        #[ink(message)]
        pub fn complete_grid_event(&mut self, event_id: u64) -> Result<(), String> {
            if self.ensure_authorized().is_err() {
                return Err("Unauthorized caller".into());
            }

            let mut event = self.events.get(event_id)
                .ok_or("Event not found")?;

            if event.completed {
                return Err("Event already completed".into());
            }

            event.active = false;
            event.completed = true;
            self.events.insert(event_id, &event);

            self.env().emit_event(GridEventCompleted {
                event_id,
                total_participants: event.total_participants,
                total_energy_reduced: event.total_energy_reduced,
            });

            Ok(())
        }

        /// Get active events
        #[ink(message)]
        pub fn get_active_events(&self) -> Vec<(u64, GridEvent)> {
            let mut active_events = Vec::new();
            let current_time = self.env().block_timestamp();
            
            // Note: This is a simplified implementation
            // In a real scenario, you'd want to iterate through events more efficiently
            for i in 1..self.next_event_id {
                if let Some(event) = self.events.get(i) {
                    if event.active && current_time <= event.end_time {
                        active_events.push((i, event));
                    }
                }
            }
            
            active_events
        }

        /// Calculate reward for participation
        fn calculate_reward(&self, event: &GridEvent, actual_reduction: u64) -> Balance {
            // Base reward calculation
            let base_reward = event.base_compensation_rate
                .saturating_mul(actual_reduction as u128)
                .saturating_div(1000); // Per kWh basis

            // Apply efficiency bonus if exceeded target
            if actual_reduction > event.target_reduction_kw {
                base_reward.saturating_mul(12).saturating_div(10) // 20% bonus
            } else {
                base_reward
            }
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

        /// Get contract statistics
        #[ink(message)]
        pub fn get_stats(&self) -> (u64, u64) {
            let total_events = self.next_event_id.saturating_sub(1);
            let mut completed_events: u64 = 0;
            
            for i in 1..self.next_event_id {
                if let Some(event) = self.events.get(i) {
                    if event.completed {
                        completed_events = completed_events.saturating_add(1);
                    }
                }
            }
            
            (total_events, completed_events)
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

        /// Update token contract address (owner only)
        #[ink(message)]
        pub fn update_token_address(&mut self, new_address: AccountId) -> Result<(), String> {
            if self.env().caller() != self.owner {
                return Err("Only owner can update token address".into());
            }
            
            self.token_address = new_address;
            Ok(())
        }

        /// Update registry contract address (owner only)
        #[ink(message)]
        pub fn update_registry_address(&mut self, new_address: AccountId) -> Result<(), String> {
            if self.env().caller() != self.owner {
                return Err("Only owner can update registry address".into());
            }
            
            self.registry_address = new_address;
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use powergrid_shared::DeviceType;
        use ink::env::test::{default_accounts, set_caller, set_block_timestamp, DefaultAccounts};
        use ink::env::DefaultEnvironment;

        #[ink::test]
        fn test_grid_event_creation() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            let mut grid_service = GridService::new(accounts.bob, accounts.charlie);

            // Test creating event as owner
            let result = grid_service.create_grid_event(
                GridEventType::DemandResponse,
                60, // 1 hour
                1000, // 1000 units per kWh
                100,  // 100 kW target
            );

            assert!(result.is_ok());
            let event_id = result.unwrap();
            assert_eq!(event_id, 1);

            let event = grid_service.get_grid_event(event_id).unwrap();
            assert_eq!(event.target_reduction_kw, 100);
        }

        #[ink::test]
        fn test_participation() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            let mut grid_service = GridService::new(accounts.bob, accounts.charlie);

            // Create event
            let event_id = grid_service.create_grid_event(
                GridEventType::PeakShaving,
                30,
                500,
                50,
            ).unwrap();

            // Participate as different user
            set_caller::<DefaultEnvironment>(accounts.alice);
            let result = grid_service.participate_in_event(event_id, 75);
            assert!(result.is_ok());

            let participations = grid_service.get_event_participations(event_id);
            assert_eq!(participations.len(), 1);
            assert_eq!(participations[0].energy_contributed_wh, 75);
        }

        #[ink::test]
        fn test_participation_verification() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            let mut grid_service = GridService::new(accounts.bob, accounts.charlie);

            // Create event
            let event_id = grid_service.create_grid_event(
                GridEventType::FrequencyRegulation,
                45,
                750,
                80,
            ).unwrap();

            // Participate
            set_caller::<DefaultEnvironment>(accounts.alice);
            let _ = grid_service.participate_in_event(event_id, 60);

            // Verify as owner
            set_caller::<DefaultEnvironment>(accounts.alice); // Reset to owner
            let result = grid_service.verify_participation(event_id, accounts.alice, 65);
            assert!(result.is_ok());

            let participations = grid_service.get_event_participations(event_id);
            assert_eq!(participations.len(), 1);
            assert!(participations[0].verified);
            assert_eq!(participations[0].energy_contributed_wh, 65);
        }
    }
}