#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod grid_service {
    use ink::prelude::{string::String, vec::Vec, format};
    use ink::storage::Mapping;
    use ink::env::call::FromAccountId;
    use powergrid_shared::{GridEvent, GridEventType, Participation, GridSignal, ink_account_to_bytes};
    use powergrid_token::powergrid_token::PowergridTokenRef;
    use resource_registry::resource_registry::ResourceRegistryRef;

    /// Grid condition monitoring data
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct GridCondition {
        pub timestamp: u64,
        pub load_mw: u64,          // Current grid load in MW
        pub capacity_mw: u64,      // Total capacity in MW
        pub frequency_hz: u32,     // Grid frequency (typically ~50Hz)
        pub voltage_kv: u32,       // Voltage level in kV
        pub renewable_percentage: u8, // % of renewable energy
    }

    /// Automatic trigger rules for grid events
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct AutoTriggerRule {
        pub rule_id: u64,
        pub active: bool,
        pub event_type: GridEventType,
        pub load_threshold_percentage: u8,  // Trigger when load > X% of capacity
        pub frequency_low_threshold: u32,   // Trigger when frequency < X (in 0.01 Hz)
        pub frequency_high_threshold: u32,  // Trigger when frequency > X (in 0.01 Hz)
        pub compensation_rate: Balance,
        pub target_reduction_percentage: u8, // % reduction target
        pub duration_minutes: u64,
    }

    /// Energy flexibility score components
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct FlexibilityScore {
        pub device: AccountId,
        pub response_time_seconds: u64,     // How quickly device responds
        pub consistency_percentage: u8,     // Historical reliability (0-100)
        pub flexibility_range_kw: u64,      // Max power adjustment capability
        pub availability_hours_per_day: u8, // Hours available for grid services
        pub total_score: u16,               // Calculated total score (0-1000)
        pub last_updated: u64,
    }

    /// Parameters for creating trigger rules to avoid too many function arguments
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct TriggerRuleParams {
        pub event_type: GridEventType,
        pub load_threshold_percentage: u8,
        pub frequency_low_threshold: u32,
        pub frequency_high_threshold: u32,
        pub compensation_rate: Balance,
        pub target_reduction_percentage: u8,
        pub duration_minutes: u64,
    }

    /// The GridService contract
    #[ink(storage)]
    pub struct GridService {
        /// Simple reentrancy flag
        entered: bool,
        /// Pause flag
        paused: bool,
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
        /// Authorized callers map
        authorized_callers: Mapping<AccountId, bool>,
        /// Base compensation rate that governance can adjust
        default_compensation_rate: Balance,
        /// Governance contract address allowed to manage roles/params
        governance_address: AccountId,
        /// Current grid conditions (updated by external feeds)
        current_grid_condition: Option<GridCondition>,
        /// Automatic trigger rules mapping
        trigger_rules: Mapping<u64, AutoTriggerRule>,
        /// Next trigger rule ID
        next_rule_id: u64,
        /// Device flexibility scores
        flexibility_scores: Mapping<AccountId, FlexibilityScore>,
        /// Grid data feed addresses (authorized to update conditions)
        data_feed_addresses: Mapping<AccountId, bool>,
        /// Auto-triggering enabled flag
        auto_trigger_enabled: bool,
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
    pub struct RewardPaid {
        #[ink(topic)]
        event_id: u64,
        #[ink(topic)]
        participant: AccountId,
        amount: Balance,
    }

    /// New automation events
    #[ink(event)]
    pub struct GridConditionUpdated {
        #[ink(topic)]
        timestamp: u64,
        load_mw: u64,
        capacity_mw: u64,
        frequency_hz: u32,
        load_percentage: u8,
    }

    #[ink(event)]
    pub struct AutoEventTriggered {
        #[ink(topic)]
        event_id: u64,
        #[ink(topic)]
        rule_id: u64,
        trigger_reason: String,
        load_percentage: u8,
        frequency_hz: u32,
    }

    #[ink(event)]
    pub struct FlexibilityScoreUpdated {
        #[ink(topic)]
        device: AccountId,
        old_score: u16,
        new_score: u16,
        response_time: u64,
        consistency: u8,
    }

    #[ink(event)]
    pub struct TriggerRuleCreated {
        #[ink(topic)]
        rule_id: u64,
        event_type: GridEventType,
        load_threshold: u8,
        frequency_low: u32,
        frequency_high: u32,
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
                entered: false,
                paused: false,
                owner: Self::env().caller(),
                token_address,
                registry_address,
                events: Mapping::default(),
                participations: Mapping::default(),
                next_event_id: 1,
                authorized_callers: Mapping::default(),
                default_compensation_rate: 0,
                governance_address: Self::env().caller(),
                current_grid_condition: None,
                trigger_rules: Mapping::default(),
                next_rule_id: 1,
                flexibility_scores: Mapping::default(),
                data_feed_addresses: Mapping::default(),
                auto_trigger_enabled: true,
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
            if self.paused { return Err("Paused".into()); }
            if self.ensure_authorized().is_err() {
                return Err("Unauthorized caller".into());
            }

            self.create_grid_event_internal(event_type, duration_minutes, compensation_rate, target_reduction_kw)
        }

        /// Internal method to create grid events (bypasses authorization for auto-triggers)
        fn create_grid_event_internal(
            &mut self,
            event_type: GridEventType,
            duration_minutes: u64,
            compensation_rate: Balance,
            target_reduction_kw: u64,
        ) -> Result<u64, String> {
            let now = self.env().block_timestamp();
            let event_id = self.next_event_id;
            
            let event = GridEvent {
                event_type: event_type.clone(),
                duration_minutes,
                base_compensation_rate: if compensation_rate > 0 { compensation_rate } else { self.default_compensation_rate },
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
            if self.entered { return Err("Reentrancy".into()); }
            self.entered = true;
            if self.paused { self.entered = false; return Err("Paused".into()); }
            let caller = self.env().caller();
            let caller_bytes = ink_account_to_bytes(caller);
            
            // Verify event exists and is active
            let mut event = self.events.get(event_id)
                .ok_or("Event not found")?;
            
            if !event.active { self.entered = false; return Err("Event is not active".into()); }

            let now = self.env().block_timestamp();
            if now > event.end_time { self.entered = false; return Err("Event has ended".into()); }

            // Verify device is registered and active in registry (skipped in unit tests)
            #[cfg(not(test))]
            {
                let registry = ResourceRegistryRef::from_account_id(self.registry_address);
                if !registry.is_device_registered(caller) {
                    return Err("Device not registered in registry".into());
                }
            }

            // Create participation record
            let participation = Participation {
                participant: caller_bytes,
                energy_contributed_wh: energy_reduction_wh,
                participation_start: now,
                participation_end: 0, // Will be set when verified
                reward_earned: 0,    // Will be calculated when verified
                verified: false,
                paid: false,
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
            self.entered = false;
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
            if self.entered { return Err("Reentrancy".into()); }
            self.entered = true;
            if self.paused { self.entered = false; return Err("Paused".into()); }
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
                    // Prevent double payout
                    if participation.verified && participation.paid {
                        return Err("AlreadyVerifiedAndPaid".into());
                    }
                    participation.energy_contributed_wh = actual_reduction;
                    participation.participation_end = self.env().block_timestamp();
                    participation.verified = true;
                    
                    // Calculate reward (includes flexibility scoring)
                    participation.reward_earned = self.calculate_reward(&event, actual_reduction, participant);
                    
                    found = true;
                    break;
                }
            }

            if !found { return Err("Participation not found".into()); }

            self.participations.insert(event_id, &participations);

            // Find the updated participation for the reward amount
            let mut reward_earned = participations.iter()
                .find(|p| p.participant == participant_bytes)
                .map(|p| p.reward_earned)
                .unwrap_or(0);
            
            // Reputation-based multiplier (80% - 120%) applied to reward; only when not testing
            #[cfg(not(test))]
            {
                let registry = ResourceRegistryRef::from_account_id(self.registry_address);
                if let Some(rep) = registry.get_device_reputation(participant) {
                    let rep_u128 = rep as u128;
                    // multiplier in basis points: 8000 + rep*40 (rep 0..=100 -> 0.8x..=1.2x)
                    let multiplier_bp: u128 = 8000u128.saturating_add(rep_u128.saturating_mul(40));
                    reward_earned = reward_earned
                        .saturating_mul(multiplier_bp)
                        .saturating_div(10_000);
                }
            }

        // Interact with token to mint rewards and update registry (skipped in unit tests)
            #[cfg(not(test))]
            {
                if reward_earned > 0 {
                    let mut token = PowergridTokenRef::from_account_id(self.token_address);
                    // Minting will succeed only if this contract is a minter; assume governance sets it
                    let _ = token.mint(participant, reward_earned);
            self.env().emit_event(RewardPaid { event_id, participant, amount: reward_earned });
                    // Mark paid
                    if let Some(p) = participations.iter_mut().find(|p| p.participant == participant_bytes) {
                        p.paid = true;
                    }
                    self.participations.insert(event_id, &participations);
                }

                let mut registry = ResourceRegistryRef::from_account_id(self.registry_address);
                let _ = registry.update_device_performance(participant, actual_reduction, true);
            }

            self.env().emit_event(ParticipationVerified {
                event_id,
                participant,
                reward_earned,
                verified: true,
            });
            self.entered = false;
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

        /// Calculate reward for participation (now includes flexibility scoring)
    fn calculate_reward(&self, event: &GridEvent, actual_reduction: u64, participant: AccountId) -> Balance {
            // Base reward calculation
            let base_reward = event.base_compensation_rate
                .saturating_mul(actual_reduction as u128)
                .saturating_div(1000); // Per kWh basis

            // Apply efficiency bonus if exceeded target
            let efficiency_reward = if actual_reduction > event.target_reduction_kw {
                base_reward.saturating_mul(12).saturating_div(10) // 20% bonus
            } else {
                base_reward
            };

            // Apply flexibility score multiplier (50% to 150% based on score)
            let flexibility_multiplier = if let Some(score) = self.flexibility_scores.get(participant) {
                // Score ranges 0-1000, convert to multiplier 500-1500 (50%-150%)
                let multiplier_bp = 500_u128.saturating_add((score.total_score as u128).saturating_mul(1000).saturating_div(1000));
                multiplier_bp.clamp(500, 1500) // Clamp between 50% and 150%
            } else {
                1000 // Default 100% if no flexibility score
            };

            efficiency_reward
                .saturating_mul(flexibility_multiplier)
                .saturating_div(1000)
        }

        /// Ingest a grid signal from an oracle/aggregator and create/complete events (authorized only)
        #[ink(message)]
        pub fn ingest_grid_signal(&mut self, signal: GridSignal) -> Result<Option<u64>, String> {
            if self.ensure_authorized().is_err() {
                return Err("Unauthorized caller".into());
            }

            let mut created = None;
            if signal.start {
                // Derive compensation from severity (1-5) times default rate
                let severity = signal.severity.clamp(1, 5) as u128;
                let rate = self.default_compensation_rate.saturating_mul(severity);
                let id = self.create_grid_event(signal.event_type, signal.duration_minutes, rate, signal.target_reduction_kw)?;
                created = Some(id);
            }

            if let Some(eid) = signal.complete_event_id {
                // Best-effort completion
                let _ = self.complete_grid_event(eid);
            }

            Ok(created)
        }

        /// Get default/base compensation rate
        #[ink(message)]
        pub fn get_default_compensation_rate(&self) -> Balance { self.default_compensation_rate }

        /// Add authorized caller (owner only)
        #[ink(message)]
        pub fn add_authorized_caller(&mut self, caller: AccountId) -> Result<(), String> {
            let sender = self.env().caller();
            if sender != self.owner && sender != self.governance_address {
                return Err("Only owner/governance can add authorized callers".into());
            }
            
            self.authorized_callers.insert(caller, &true);
            Ok(())
        }

        /// Remove authorized caller (owner only)
        #[ink(message)]
        pub fn remove_authorized_caller(&mut self, caller: AccountId) -> Result<(), String> {
            let sender = self.env().caller();
            if sender != self.owner && sender != self.governance_address {
                return Err("Only owner/governance can remove authorized callers".into());
            }
            
            self.authorized_callers.remove(caller);
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
            if caller == self.owner || caller == self.governance_address || self.authorized_callers.get(caller).unwrap_or(false) {
                Ok(())
            } else {
                Err("Unauthorized caller".into())
            }
        }

        /// Update token contract address (owner only)
        #[ink(message)]
        pub fn update_token_address(&mut self, new_address: AccountId) -> Result<(), String> {
            if self.env().caller() != self.owner && self.env().caller() != self.governance_address {
                return Err("Only owner can update token address".into());
            }
            
            self.token_address = new_address;
            Ok(())
        }

        /// Update registry contract address (owner only)
        #[ink(message)]
        pub fn update_registry_address(&mut self, new_address: AccountId) -> Result<(), String> {
            if self.env().caller() != self.owner && self.env().caller() != self.governance_address {
                return Err("Only owner can update registry address".into());
            }
            
            self.registry_address = new_address;
            Ok(())
        }

        /// Set governance address (owner only)
        #[ink(message)]
        pub fn set_governance_address(&mut self, addr: AccountId) -> Result<(), String> {
            if self.env().caller() != self.owner {
                return Err("Only owner can set governance address".into());
            }
            self.governance_address = addr;
            Ok(())
        }

        /// Update the default/base compensation rate (owner or authorized)
        #[ink(message)]
        pub fn update_default_compensation_rate(&mut self, new_rate: Balance) -> Result<(), String> {
            if self.ensure_authorized().is_err() {
                return Err("Unauthorized caller".into());
            }
            self.default_compensation_rate = new_rate;
            Ok(())
        }

        /// Pause/unpause admin (owner or governance)
        #[ink(message)]
    pub fn set_paused(&mut self, pause: bool) -> Result<(), String> {
            let sender = self.env().caller();
            if sender != self.owner && sender != self.governance_address { return Err("Unauthorized".into()); }
        self.paused = pause;
            Ok(())
        }

        // === GRID AUTOMATION FUNCTIONS ===

        /// Update grid conditions (data feed only)
        #[ink(message)]
        pub fn update_grid_condition(
            &mut self,
            load_mw: u64,
            capacity_mw: u64,
            frequency_hz: u32,
            voltage_kv: u32,
            renewable_percentage: u8,
        ) -> Result<(), String> {
            let caller = self.env().caller();
            if !self.data_feed_addresses.get(caller).unwrap_or(false) && caller != self.owner {
                return Err("Unauthorized data feed".into());
            }

            let timestamp = self.env().block_timestamp();
            let condition = GridCondition {
                timestamp,
                load_mw,
                capacity_mw,
                frequency_hz,
                voltage_kv,
                renewable_percentage,
            };

            let load_percentage = if capacity_mw > 0 {
                match load_mw.checked_mul(100) {
                    Some(load_times_100) => {
                        match load_times_100.checked_div(capacity_mw) {
                            Some(percentage) => {
                                if percentage > 100 { 
                                    100u8 
                                } else { 
                                    u8::try_from(percentage).unwrap_or(100u8)
                                }
                            },
                            None => 0u8,
                        }
                    },
                    None => 100u8, // overflow means very high load, cap at 100%
                }
            } else {
                0u8
            };

            self.current_grid_condition = Some(condition.clone());

            self.env().emit_event(GridConditionUpdated {
                timestamp,
                load_mw,
                capacity_mw,
                frequency_hz,
                load_percentage,
            });

            // Check auto-trigger rules
            if self.auto_trigger_enabled {
                self.check_auto_triggers(load_percentage, frequency_hz)?;
            }

            Ok(())
        }

        /// Check and trigger automatic grid events based on conditions
        fn check_auto_triggers(&mut self, load_percentage: u8, frequency_hz: u32) -> Result<(), String> {
            let mut triggered_rules = Vec::new();
            
            // Collect all active rules that should trigger
            for rule_id in 1..self.next_rule_id {
                if let Some(rule) = self.trigger_rules.get(rule_id) {
                    if !rule.active { continue; }

                    let should_trigger = 
                        load_percentage >= rule.load_threshold_percentage ||
                        frequency_hz < rule.frequency_low_threshold ||
                        frequency_hz > rule.frequency_high_threshold;

                    if should_trigger {
                        triggered_rules.push((rule_id, rule));
                    }
                }
            }

            // Trigger events for matching rules
            for (rule_id, rule) in triggered_rules {
                let target_reduction_kw = if let Some(condition) = &self.current_grid_condition {
                    condition.load_mw.saturating_mul(1000).saturating_mul(rule.target_reduction_percentage as u64).saturating_div(100)
                } else {
                    1000 // Default 1MW target
                };

                let trigger_reason = if load_percentage >= rule.load_threshold_percentage {
                    format!("High load: {}%", load_percentage)
                } else if frequency_hz < rule.frequency_low_threshold {
                    format!("Low frequency: {}.{:02}Hz", frequency_hz.saturating_div(100), frequency_hz % 100)
                } else {
                    format!("High frequency: {}.{:02}Hz", frequency_hz.saturating_div(100), frequency_hz % 100)
                };

                // Create the event
                match self.create_grid_event_internal(
                    rule.event_type.clone(),
                    rule.duration_minutes,
                    rule.compensation_rate,
                    target_reduction_kw,
                ) {
                    Ok(event_id) => {
                        self.env().emit_event(AutoEventTriggered {
                            event_id,
                            rule_id,
                            trigger_reason,
                            load_percentage,
                            frequency_hz,
                        });
                    }
                    Err(_) => {
                        // Failed to create event, continue with other rules
                        continue;
                    }
                }
            }

            Ok(())
        }

        /// Create an automatic trigger rule (owner/governance only)
        #[ink(message)]
        pub fn create_trigger_rule(
            &mut self,
            params: TriggerRuleParams,
        ) -> Result<u64, String> {
            let caller = self.env().caller();
            if caller != self.owner && caller != self.governance_address {
                return Err("Unauthorized".into());
            }

            let rule_id = self.next_rule_id;
            let rule = AutoTriggerRule {
                rule_id,
                active: true,
                event_type: params.event_type.clone(),
                load_threshold_percentage: params.load_threshold_percentage,
                frequency_low_threshold: params.frequency_low_threshold,
                frequency_high_threshold: params.frequency_high_threshold,
                compensation_rate: params.compensation_rate,
                target_reduction_percentage: params.target_reduction_percentage,
                duration_minutes: params.duration_minutes,
            };

            self.trigger_rules.insert(rule_id, &rule);
            self.next_rule_id = self.next_rule_id.saturating_add(1);

            self.env().emit_event(TriggerRuleCreated {
                rule_id,
                event_type: params.event_type,
                load_threshold: params.load_threshold_percentage,
                frequency_low: params.frequency_low_threshold,
                frequency_high: params.frequency_high_threshold,
            });

            Ok(rule_id)
        }

        /// Update device flexibility score (registry or owner only)
        #[ink(message)]
        pub fn update_flexibility_score(
            &mut self,
            device: AccountId,
            response_time_seconds: u64,
            consistency_percentage: u8,
            flexibility_range_kw: u64,
            availability_hours_per_day: u8,
        ) -> Result<(), String> {
            let caller = self.env().caller();
            if caller != self.registry_address && caller != self.owner {
                return Err("Unauthorized".into());
            }

            let old_score = self.flexibility_scores.get(device)
                .map(|s| s.total_score)
                .unwrap_or(0);

            // Calculate total flexibility score (0-1000 scale)
            let response_score: u16 = if response_time_seconds <= 60 { 250 } // Excellent: ≤1 min
                else if response_time_seconds <= 300 { 200 }           // Good: ≤5 min
                else if response_time_seconds <= 900 { 150 }           // Fair: ≤15 min
                else { 100 };                                          // Poor: >15 min

            let consistency_score = (consistency_percentage as u16).saturating_mul(250).saturating_div(100); // 0-250 based on %

            let flexibility_score: u16 = if flexibility_range_kw >= 100 { 250 }      // Excellent: ≥100kW
                else if flexibility_range_kw >= 50 { 200 }                      // Good: ≥50kW
                else if flexibility_range_kw >= 10 { 150 }                      // Fair: ≥10kW
                else { 100 };                                                   // Poor: <10kW

            let availability_score = (availability_hours_per_day as u16).saturating_mul(250).saturating_div(24); // 0-250 based on hours

            let total_score = response_score.saturating_add(consistency_score).saturating_add(flexibility_score).saturating_add(availability_score);

            let score = FlexibilityScore {
                device,
                response_time_seconds,
                consistency_percentage,
                flexibility_range_kw,
                availability_hours_per_day,
                total_score,
                last_updated: self.env().block_timestamp(),
            };

            self.flexibility_scores.insert(device, &score);

            self.env().emit_event(FlexibilityScoreUpdated {
                device,
                old_score,
                new_score: total_score,
                response_time: response_time_seconds,
                consistency: consistency_percentage,
            });

            Ok(())
        }

        /// Add authorized data feed address (owner only)
        #[ink(message)]
        pub fn add_data_feed(&mut self, feed_address: AccountId) -> Result<(), String> {
            if self.env().caller() != self.owner {
                return Err("Unauthorized".into());
            }
            self.data_feed_addresses.insert(feed_address, &true);
            Ok(())
        }

        /// Get current grid condition
        #[ink(message)]
        pub fn get_grid_condition(&self) -> Option<GridCondition> {
            self.current_grid_condition.clone()
        }

        /// Get device flexibility score
        #[ink(message)]
        pub fn get_flexibility_score(&self, device: AccountId) -> Option<FlexibilityScore> {
            self.flexibility_scores.get(device)
        }

        /// Get trigger rule
        #[ink(message)]
        pub fn get_trigger_rule(&self, rule_id: u64) -> Option<AutoTriggerRule> {
            self.trigger_rules.get(rule_id)
        }

        /// Enable/disable auto-triggering (owner only)
        #[ink(message)]
        pub fn set_auto_trigger_enabled(&mut self, enabled: bool) -> Result<(), String> {
            if self.env().caller() != self.owner {
                return Err("Unauthorized".into());
            }
            self.auto_trigger_enabled = enabled;
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

        #[ink::test]
        fn test_grid_automation_system() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            let mut grid_service = GridService::new(accounts.bob, accounts.charlie);

            // Test 1: Add data feed authorization
            let result = grid_service.add_data_feed(accounts.django);
            assert!(result.is_ok());

            // Test 2: Create an auto-trigger rule
            let rule_params = TriggerRuleParams {
                event_type: GridEventType::Emergency,
                load_threshold_percentage: 85, // Load threshold 85%
                frequency_low_threshold: 4950, // Low frequency 49.50 Hz
                frequency_high_threshold: 5050, // High frequency 50.50 Hz
                compensation_rate: 1000, // Compensation rate
                target_reduction_percentage: 10, // 10% reduction target
                duration_minutes: 30, // 30 minutes duration
            };
            let rule_result = grid_service.create_trigger_rule(rule_params);
            assert!(rule_result.is_ok());
            let rule_id = rule_result.unwrap();
            assert_eq!(rule_id, 1);

            // Test 3: Verify rule was created
            let rule = grid_service.get_trigger_rule(rule_id);
            assert!(rule.is_some());
            let rule = rule.unwrap();
            assert_eq!(rule.load_threshold_percentage, 85);
            assert_eq!(rule.event_type, GridEventType::Emergency);

            // Test 4: Update grid conditions (should NOT trigger - below threshold)
            set_caller::<DefaultEnvironment>(accounts.django);
            let result = grid_service.update_grid_condition(
                800,  // 800 MW load
                1000, // 1000 MW capacity (80% load - below 85% threshold)
                5000, // 50.00 Hz (normal frequency)
                400,  // 400 kV
                30,   // 30% renewable
            );
            assert!(result.is_ok());

            // Test 5: Verify grid condition was stored
            let condition = grid_service.get_grid_condition();
            assert!(condition.is_some());
            let condition = condition.unwrap();
            assert_eq!(condition.load_mw, 800);
            assert_eq!(condition.capacity_mw, 1000);

            // Test 6: Update with high load (should trigger auto-event)
            let result = grid_service.update_grid_condition(
                870,  // 870 MW load  
                1000, // 1000 MW capacity (87% load - above 85% threshold)
                5000, // 50.00 Hz
                400,  // 400 kV  
                25,   // 25% renewable
            );
            assert!(result.is_ok());

            // Test 7: Check that auto-event was created (next_event_id should be 2)
            assert_eq!(grid_service.next_event_id, 2);

            // Test 8: Update with low frequency (should trigger another auto-event)
            let result = grid_service.update_grid_condition(
                800,  // 800 MW load (80% - below threshold)
                1000, // 1000 MW capacity
                4940, // 49.40 Hz (below 49.50 threshold)
                400,  // 400 kV
                30,   // 30% renewable
            );
            assert!(result.is_ok());

            // Test 9: Check that another auto-event was created
            assert_eq!(grid_service.next_event_id, 3);
        }

        #[ink::test]
        fn test_flexibility_scoring() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            let mut grid_service = GridService::new(accounts.bob, accounts.charlie);

            // Test 1: Update flexibility score for a device
            let result = grid_service.update_flexibility_score(
                accounts.alice,
                45,  // 45 seconds response time (excellent)
                85,  // 85% consistency (good)
                120, // 120 kW flexibility range (excellent)
                20,  // 20 hours availability per day (good)
            );
            assert!(result.is_ok());

            // Test 2: Verify score was calculated correctly
            let score = grid_service.get_flexibility_score(accounts.alice);
            assert!(score.is_some());
            let score = score.unwrap();
            
            // Expected calculation:
            // Response: 45s ≤ 60s = 250 points (excellent)
            // Consistency: 85% = 85 * 250 / 100 = 212 points
            // Flexibility: 120kW ≥ 100kW = 250 points (excellent)  
            // Availability: 20h = 20 * 250 / 24 = 208 points
            // Total: 250 + 212 + 250 + 208 = 920 points
            assert_eq!(score.total_score, 920);
            assert_eq!(score.response_time_seconds, 45);
            assert_eq!(score.consistency_percentage, 85);
            assert_eq!(score.flexibility_range_kw, 120);

            // Test 3: Update with poor performance metrics
            let result = grid_service.update_flexibility_score(
                accounts.bob,
                1200, // 20 minutes response time (poor)
                40,   // 40% consistency (poor)
                5,    // 5 kW flexibility (poor)
                8,    // 8 hours availability (poor)
            );
            assert!(result.is_ok());

            let score = grid_service.get_flexibility_score(accounts.bob);
            assert!(score.is_some());
            let score = score.unwrap();
            
            // Expected calculation:
            // Response: 1200s > 900s = 100 points (poor)
            // Consistency: 40% = 40 * 250 / 100 = 100 points
            // Flexibility: 5kW < 10kW = 100 points (poor)
            // Availability: 8h = 8 * 250 / 24 = 83 points
            // Total: 100 + 100 + 100 + 83 = 383 points
            assert_eq!(score.total_score, 383);
        }

        #[ink::test]
        fn test_enhanced_reward_calculation() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            let mut grid_service = GridService::new(accounts.bob, accounts.charlie);

            // Test 1: Set up a device with excellent flexibility score
            let result = grid_service.update_flexibility_score(
                accounts.alice,
                30,  // 30s response (excellent)
                95,  // 95% consistency (excellent)
                150, // 150kW range (excellent)
                24,  // 24h availability (excellent)
            );
            assert!(result.is_ok());

            // Test 2: Create a grid event
            let event_id = grid_service.create_grid_event(
                GridEventType::DemandResponse,
                60,
                1000, // 1000 base compensation
                100,  // 100 kW target
            ).unwrap();

            // Test 3: Participate in event
            set_caller::<DefaultEnvironment>(accounts.alice);
            let _ = grid_service.participate_in_event(event_id, 120); // Contribute 120 kW

            // Test 4: Verify participation with enhanced rewards
            set_caller::<DefaultEnvironment>(accounts.alice); // Reset to owner
            let result = grid_service.verify_participation(event_id, accounts.alice, 120);
            assert!(result.is_ok());

            // Test 5: Check that reward was enhanced based on flexibility score
            let participations = grid_service.get_event_participations(event_id);
            assert_eq!(participations.len(), 1);
            assert!(participations[0].verified);
            
            // Base reward: 1000 * 120 / 1000 = 120
            // Efficiency bonus: 120 * 1.2 = 144 (exceeded target of 100)
            // Flexibility multiplier: excellent score should give ~150% = 216
            // Final reward should be higher than base due to flexibility scoring
            assert!(participations[0].reward_earned > 144);
        }
    }
}