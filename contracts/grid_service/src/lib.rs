#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod grid_service {
    use powergrid_shared::{
    GridEvent, GridEventType, Participation, GridServiceInterface,
};
    use ink::storage::Mapping;
    use ink::prelude::{string::String, vec::Vec};

    #[ink(storage)]
    pub struct GridService {
        // Contract addresses
        token_address: AccountId,
        registry_address: AccountId,
        admin: AccountId,
        governance_address: Option<AccountId>,
        
        // Events management
        events: Mapping<u64, GridEvent>,
        event_count: u64,
        
        // Participation tracking
        event_participants: Mapping<(u64, AccountId), Participation>,
        participant_count: Mapping<u64, u32>,
        
        // Reward calculation parameters
        base_reward_multiplier: u32, // Multiplier for base rewards (in basis points)
        reputation_bonus_threshold: u32, // Minimum reputation for bonus
        reputation_bonus_multiplier: u32, // Bonus multiplier (in basis points)
        
        // Energy tracking
        total_energy_saved: u64,
        total_rewards_distributed: Balance,
    }

    #[ink(event)]
    pub struct GridEventCreated {
        #[ink(topic)]
        event_id: u64,
        event_type: GridEventType,
        duration_minutes: u64,
        target_reduction_kw: u64,
        compensation_rate: Balance,
    }

    #[ink(event)]
    pub struct ParticipationRegistered {
        #[ink(topic)]
        event_id: u64,
        #[ink(topic)]
        participant: AccountId,
        energy_contributed: u64,
        reward_earned: Balance,
    }

    #[ink(event)]
    pub struct GridEventCompleted {
        #[ink(topic)]
        event_id: u64,
        total_participants: u32,
        total_energy_reduced: u64,
        total_rewards: Balance,
    }

    #[ink(event)]
    pub struct RewardsDistributed {
        #[ink(topic)]
        event_id: u64,
        total_amount: Balance,
        participant_count: u32,
    }

    impl GridService {
        #[ink(constructor)]
        pub fn new(token_address: AccountId, registry_address: AccountId) -> Self {
            Self {
                token_address,
                registry_address,
                admin: Self::env().caller(),
                governance_address: None,
                events: Mapping::default(),
                event_count: 0,
                event_participants: Mapping::default(),
                participant_count: Mapping::default(),
                base_reward_multiplier: 10000, // 100% in basis points
                reputation_bonus_threshold: 80,
                reputation_bonus_multiplier: 2000, // 20% bonus in basis points
                total_energy_saved: 0,
                total_rewards_distributed: 0,
            }
        }

        #[ink(message)]
        pub fn create_grid_event(
            &mut self,
            event_type: GridEventType,
            duration_minutes: u64,
            base_compensation_rate: Balance,
            target_reduction_kw: u64,
            start_delay_minutes: u64,
        ) -> Result<u64, String> {
            self.ensure_admin_or_governance()?;

            let current_time = self.env().block_timestamp();
            let start_time = current_time + (start_delay_minutes * 60 * 1000); // Convert to milliseconds
            let end_time = start_time + (duration_minutes * 60 * 1000);

            let event_id = self.event_count;
            let event = GridEvent {
                event_type: event_type.clone(),
                duration_minutes,
                base_compensation_rate,
                target_reduction_kw,
                created_at: current_time,
                start_time,
                end_time,
                active: true,
                total_participants: 0,
                total_energy_reduced: 0,
                completed: false,
            };

            self.events.insert(event_id, &event);
            self.event_count += 1;
            self.participant_count.insert(event_id, &0);

            self.env().emit_event(GridEventCreated {
                event_id,
                event_type,
                duration_minutes,
                target_reduction_kw,
                compensation_rate: base_compensation_rate,
            });

            Ok(event_id)
        }

        #[ink(message)]
        pub fn register_participation(&mut self, event_id: u64) -> Result<(), String> {
            let caller = self.env().caller();
            let current_time = self.env().block_timestamp();
            
            let event = self.events.get(event_id)
                .ok_or("Event not found")?;

            // Validate event status
            if !event.active {
                return Err("Event is not active".into());
            }

            if current_time < event.start_time {
                return Err("Event has not started yet".into());
            }

            if current_time > event.end_time {
                return Err("Event has ended".into());
            }

            // Check if already participating
            if self.event_participants.contains((event_id, caller)) {
                return Err("Already registered for this event".into());
            }

            // Verify device registration through cross-contract call
            if !self.is_device_eligible(caller)? {
                return Err("Device not registered or not eligible".into());
            }

            // Create participation record
            let participation = Participation {
                participant: caller,
                energy_contributed_wh: 0, // Will be updated when verified
                participation_start: current_time,
                participation_end: 0,
                reward_earned: 0,
                verified: false,
            };

            self.event_participants.insert((event_id, caller), &participation);
            
            // Update participant count
            let current_count = self.participant_count.get(event_id).unwrap_or(0);
            self.participant_count.insert(event_id, &(current_count + 1));

            Ok(())
        }

        #[ink(message)]
        pub fn verify_and_record_participation(
            &mut self,
            event_id: u64,
            participant: AccountId,
            energy_contributed_wh: u64,
        ) -> Result<(), String> {
            self.ensure_admin_or_governance()?;

            let mut participation = self.event_participants.get((event_id, participant))
                .ok_or("Participation not found")?;

            if participation.verified {
                return Err("Participation already verified".into());
            }

            let current_time = self.env().block_timestamp();
            participation.energy_contributed_wh = energy_contributed_wh;
            participation.participation_end = current_time;
            participation.verified = true;

            // Calculate reward
            let reward = self.calculate_participation_reward(event_id, participant, energy_contributed_wh)?;
            participation.reward_earned = reward;

            self.event_participants.insert((event_id, participant), &participation);

            // Update device performance in registry (simplified)
            // In real implementation, this would be a cross-contract call

            self.env().emit_event(ParticipationRegistered {
                event_id,
                participant,
                energy_contributed: energy_contributed_wh,
                reward_earned: reward,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn complete_grid_event(&mut self, event_id: u64) -> Result<(), String> {
            self.ensure_admin_or_governance()?;

            let mut event = self.events.get(event_id)
                .ok_or("Event not found")?;

            if event.completed {
                return Err("Event already completed".into());
            }

            let current_time = self.env().block_timestamp();
            if current_time < event.end_time {
                return Err("Event has not ended yet".into());
            }

            // Calculate total energy reduced and update event
            let (total_energy, total_rewards) = self.calculate_event_totals(event_id);
            
            event.total_energy_reduced = total_energy;
            event.completed = true;
            event.active = false;

            self.events.insert(event_id, &event);

            // Update global statistics
            self.total_energy_saved += total_energy;
            self.total_rewards_distributed += total_rewards;

            // Distribute rewards
            self.distribute_event_rewards(event_id)?;

            let participant_count = self.participant_count.get(event_id).unwrap_or(0);

            self.env().emit_event(GridEventCompleted {
                event_id,
                total_participants: participant_count,
                total_energy_reduced: total_energy,
                total_rewards,
            });

            Ok(())
        }

        // ========================================================================
        // REWARD CALCULATION ALGORITHMS
        // ========================================================================

        fn calculate_participation_reward(
            &self,
            event_id: u64,
            participant: AccountId,
            energy_contributed_wh: u64,
        ) -> Result<Balance, String> {
            let event = self.events.get(event_id)
                .ok_or("Event not found")?;

            // Base reward calculation: energy_kWh * compensation_rate
            let energy_kwh = energy_contributed_wh / 1000; // Convert Wh to kWh
            let mut base_reward = (energy_kwh as u128 * event.base_compensation_rate as u128) as Balance;

            // Apply base multiplier
            base_reward = (base_reward as u128 * self.base_reward_multiplier as u128 / 10000) as Balance;

            // Get device reputation and apply bonus (simplified)
            // In real implementation, this would be a cross-contract call
            let reputation = 85; // Simplified
            if reputation >= self.reputation_bonus_threshold {
                let bonus = (base_reward as u128 * self.reputation_bonus_multiplier as u128 / 10000) as Balance;
                base_reward += bonus;
            }

            // Event type multiplier
            let event_multiplier = match event.event_type {
                GridEventType::Emergency => 15000, // 150% for emergency events
                GridEventType::FrequencyRegulation => 12000, // 120% for frequency regulation
                GridEventType::PeakShaving => 11000, // 110% for peak shaving
                GridEventType::DemandResponse => 10000, // 100% for demand response
                GridEventType::LoadBalancing => 10500, // 105% for load balancing
            };

            base_reward = (base_reward as u128 * event_multiplier as u128 / 10000) as Balance;

            Ok(base_reward)
        }

        fn calculate_event_totals(&self, event_id: u64) -> (u64, Balance) {
            // In a real implementation, this would iterate through all participants
            // For now, we return simplified totals
            let participant_count = self.participant_count.get(event_id).unwrap_or(0);
            let estimated_energy_per_participant = 2000; // 2 kWh average
            let estimated_reward_per_participant = 100; // 100 tokens average
            
            (
                participant_count as u64 * estimated_energy_per_participant,
                participant_count as Balance * estimated_reward_per_participant,
            )
        }

        fn distribute_event_rewards(&mut self, event_id: u64) -> Result<(), String> {
            let rewards = self.calculate_all_event_rewards(event_id);
            let total_amount = rewards.iter().map(|(_, amount)| *amount).sum();
            
            // In a real implementation, this would call the token contract to mint/distribute
            // For now, we just emit an event
            
            self.env().emit_event(RewardsDistributed {
                event_id,
                total_amount,
                participant_count: rewards.len() as u32,
            });

            Ok(())
        }

        fn calculate_all_event_rewards(&self, event_id: u64) -> Vec<(AccountId, Balance)> {
            // In a real implementation, this would iterate through all event participants
            // For now, return empty vector (simplified)
            Vec::new()
        }

        // ========================================================================
        // CROSS-CONTRACT CALLS (Simplified)
        // ========================================================================

        fn is_device_eligible(&self, account: AccountId) -> Result<bool, String> {
            // In a real implementation, this would make a cross-contract call
            // For now, we assume all accounts are eligible (simplified)
            Ok(true)
        }

        // ========================================================================
        // ADMIN FUNCTIONS
        // ========================================================================

        #[ink(message)]
        pub fn set_governance_address(&mut self, governance_address: AccountId) -> Result<(), String> {
            if self.env().caller() != self.admin {
                return Err("Only admin can set governance address".into());
            }
            self.governance_address = Some(governance_address);
            Ok(())
        }

        #[ink(message)]
        pub fn update_reward_parameters(
            &mut self,
            base_multiplier: u32,
            bonus_threshold: u32,
            bonus_multiplier: u32,
        ) -> Result<(), String> {
            self.ensure_admin_or_governance()?;
            
            self.base_reward_multiplier = base_multiplier;
            self.reputation_bonus_threshold = bonus_threshold;
            self.reputation_bonus_multiplier = bonus_multiplier;
            
            Ok(())
        }

        fn ensure_admin_or_governance(&self) -> Result<(), String> {
            let caller = self.env().caller();
            if caller == self.admin || self.governance_address == Some(caller) {
                Ok(())
            } else {
                Err("Admin or governance access required".into())
            }
        }

        // ========================================================================
        // GETTER FUNCTIONS
        // ========================================================================

        #[ink(message)]
        pub fn get_event(&self, event_id: u64) -> Option<GridEvent> {
            self.events.get(event_id)
        }

        #[ink(message)]
        pub fn get_participation(&self, event_id: u64, participant: AccountId) -> Option<Participation> {
            self.event_participants.get((event_id, participant))
        }

        #[ink(message)]
        pub fn get_event_count(&self) -> u64 {
            self.event_count
        }

        #[ink(message)]
        pub fn get_total_energy_saved(&self) -> u64 {
            self.total_energy_saved
        }

        #[ink(message)]
        pub fn get_total_rewards_distributed(&self) -> Balance {
            self.total_rewards_distributed
        }

        #[ink(message)]
        pub fn get_active_events(&self) -> Vec<u64> {
            // In a real implementation, this would filter active events
            // For now, return simplified list
            Vec::new()
        }
    }

    impl GridServiceInterface for GridService {
        #[ink(message)]
        fn calculate_event_rewards(&self, event_id: u64) -> Vec<(AccountId, Balance)> {
            self.calculate_all_event_rewards(event_id)
        }

        #[ink(message)]
        fn verify_participation(&mut self, event_id: u64, participant: AccountId, energy_contributed: u64) {
            let _ = self.verify_and_record_participation(event_id, participant, energy_contributed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ink::env::test::{default_accounts, set_caller, set_block_timestamp};
    use powergrid_shared::GridEventType;

    #[ink::test]
    fn test_grid_event_creation() {
        let accounts = default_accounts();
        let mut grid_service = grid_service::GridService::new(accounts.bob, accounts.charlie);

        set_caller(accounts.alice); // Admin

        let result = grid_service.create_grid_event(
            GridEventType::DemandResponse,
            60, // 1 hour duration
            100, // 100 tokens per kWh
            5000, // 5 MW target reduction
            15, // Start in 15 minutes
        );

        assert!(result.is_ok());
        let event_id = result.unwrap();

        let event = grid_service.get_event(event_id).unwrap();
        assert_eq!(event.duration_minutes, 60);
        assert_eq!(event.base_compensation_rate, 100);
        assert_eq!(event.target_reduction_kw, 5000);
        assert!(event.active);
        assert!(!event.completed);
    }

    #[ink::test]
    fn test_grid_event_participation() {
        let accounts = default_accounts();
        let mut grid_service = grid_service::GridService::new(accounts.bob, accounts.charlie);

        // Create event
        set_caller(accounts.alice); // Admin
        let event_id = grid_service.create_grid_event(
            GridEventType::DemandResponse,
            60,
            100,
            5000,
            0, // Start immediately
        ).unwrap();

        // Simulate time passing to event start
        set_block_timestamp(1000);

        // Register participation
        set_caller(accounts.bob);
        let result = grid_service.register_participation(event_id);
        assert!(result.is_ok());

        // Verify participation record
        let participation = grid_service.get_participation(event_id, accounts.bob);
        assert!(participation.is_some());
        assert!(!participation.unwrap().verified);
    }

    #[ink::test]
    fn test_participation_verification() {
        let accounts = default_accounts();
        let mut grid_service = grid_service::GridService::new(accounts.bob, accounts.charlie);

        // Create event and register participation
        set_caller(accounts.alice);
        let event_id = grid_service.create_grid_event(
            GridEventType::DemandResponse,
            60,
            100,
            5000,
            0,
        ).unwrap();

        set_caller(accounts.bob);
        assert!(grid_service.register_participation(event_id).is_ok());

        // Verify participation
        set_caller(accounts.alice); // Admin
        let result = grid_service.verify_and_record_participation(
            event_id,
            accounts.bob,
            3000, // 3 kWh contributed
        );
        assert!(result.is_ok());

        let participation = grid_service.get_participation(event_id, accounts.bob).unwrap();
        assert!(participation.verified);
        assert_eq!(participation.energy_contributed_wh, 3000);
        assert!(participation.reward_earned > 0);
    }
}