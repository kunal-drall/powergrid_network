#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod governance {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;
    use ink::env::call::FromAccountId;
    use powergrid_shared::{Proposal, ProposalType, ink_account_to_bytes};
    use resource_registry::resource_registry::ResourceRegistryRef;
    use grid_service::grid_service::GridServiceRef;
    use powergrid_token::powergrid_token::PowergridTokenRef;

    /// The Governance contract
    #[ink(storage)]
    pub struct Governance {
        /// Simple reentrancy flag
        entered: bool,
        /// Contract owner
        owner: AccountId,
        /// Token contract for voting power
        token_address: AccountId,
        /// Registry contract for device verification
        registry_address: AccountId,
        /// Grid service contract
        grid_service_address: AccountId,
        /// Proposals mapping
        proposals: Mapping<u64, Proposal>,
        /// Voting records (proposal_id -> voter -> voted)
        #[allow(clippy::type_complexity)]
        votes: Mapping<(u64, [u8; 32]), bool>,
        /// Next proposal ID
        next_proposal_id: u64,
        /// Minimum voting power required to create proposals
        min_voting_power: Balance,
        /// Voting duration in blocks
        voting_duration_blocks: u64,
        /// Quorum percentage (out of 100)
    quorum_percentage: u32,
    /// Timelock in seconds to delay execution after queuing
    timelock_seconds: u64,
    /// Queue timestamps for proposals (proposal_id -> queued_at timestamp)
    queue_times: Mapping<u64, u64>,
    }

    /// Events emitted by the contract
    #[ink(event)]
    pub struct ProposalCreated {
        #[ink(topic)]
        proposal_id: u64,
        #[ink(topic)]
        proposer: AccountId,
        proposal_type: ProposalType,
        description: String,
        voting_end: u64,
    }

    #[ink(event)]
    pub struct VoteCast {
        #[ink(topic)]
        proposal_id: u64,
        #[ink(topic)]
        voter: AccountId,
        support: bool,
        voting_power: u64,
        reason: String,
    }

    #[ink(event)]
    pub struct ProposalExecuted {
        #[ink(topic)]
        proposal_id: u64,
        successful: bool,
    }

    #[ink(event)]
    pub struct ProposalQueued {
        #[ink(topic)]
        proposal_id: u64,
        queued_at: u64,
        execute_after: u64,
    }

    #[ink(event)]
    pub struct TimelockUpdated {
        old_seconds: u64,
        new_seconds: u64,
    }

    /// Errors
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        Unauthorized,
        ProposalNotFound,
        ProposalExpired,
        ProposalNotExpired,
        AlreadyVoted,
        InsufficientVotingPower,
        ProposalAlreadyExecuted,
        InvalidQuorum,
        InvalidDuration,
        ExecutionFailed,
        NotQueued,
        TimelockNotElapsed,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Governance {
        /// Constructor
        #[ink(constructor, payable)]
        pub fn new(
            token_address: AccountId,
            registry_address: AccountId,
            grid_service_address: AccountId,
            min_voting_power: Balance,
            voting_duration_blocks: u64,
            quorum_percentage: u32,
        ) -> Self {
            Self {
                entered: false,
                owner: Self::env().caller(),
                token_address,
                registry_address,
                grid_service_address,
                proposals: Mapping::default(),
                votes: Mapping::default(),
                next_proposal_id: 1,
                min_voting_power,
                voting_duration_blocks,
                quorum_percentage,
                timelock_seconds: 0,
                queue_times: Mapping::default(),
            }
        }

        /// Create a new proposal
        #[ink(message)]
        pub fn create_proposal(
            &mut self,
            proposal_type: ProposalType,
            description: String,
        ) -> Result<u64> {
            let caller = self.env().caller();
            let caller_bytes = ink_account_to_bytes(caller);

            // Check voting power from PSP22 balance
            let voting_power = self.get_voting_power(caller);
            if (voting_power as u128) < self.min_voting_power {
                return Err(Error::InsufficientVotingPower);
            }

            let current_block = self.env().block_number();
            let voting_end = (current_block as u64).saturating_add(self.voting_duration_blocks);
            let proposal_id = self.next_proposal_id;

            let proposal = Proposal {
                proposer: caller_bytes,
                proposal_type: proposal_type.clone(),
                description: description.clone(),
                yes_votes: 0,
                no_votes: 0,
                total_voting_power: 0,
                created_at: self.env().block_timestamp(),
                voting_end,
                executed: false,
                active: true,
            };

            self.proposals.insert(proposal_id, &proposal);
            self.next_proposal_id = self.next_proposal_id.saturating_add(1);

            self.env().emit_event(ProposalCreated {
                proposal_id,
                proposer: caller,
                proposal_type,
                description,
                voting_end,
            });

            Ok(proposal_id)
        }

        /// Vote on a proposal
        #[ink(message)]
        pub fn vote(&mut self, proposal_id: u64, support: bool, reason: String) -> Result<()> {
            if self.entered { self.entered = false; return Err(Error::Unauthorized); }
            self.entered = true;
            
            let caller = self.env().caller();
            let caller_bytes = ink_account_to_bytes(caller);

            let mut proposal = self.proposals.get(proposal_id)
                .ok_or(Error::ProposalNotFound)?;

            // Check if proposal is still active
            let current_block = self.env().block_number();
            if (current_block as u64) > proposal.voting_end { 
                self.entered = false;
                return Err(Error::ProposalExpired); 
            }

            // Check if already voted
            if self.votes.contains((proposal_id, caller_bytes)) { 
                self.entered = false;
                return Err(Error::AlreadyVoted); 
            }

            // Get voting power (simplified)
            let voting_power = self.get_voting_power(caller);
            if voting_power == 0 { 
                self.entered = false;
                return Err(Error::InsufficientVotingPower); 
            }

            // Record vote
            self.votes.insert((proposal_id, caller_bytes), &true);

            // Update proposal votes
            if support {
                proposal.yes_votes = proposal.yes_votes.saturating_add(voting_power);
            } else {
                proposal.no_votes = proposal.no_votes.saturating_add(voting_power);
            }
            proposal.total_voting_power = proposal.total_voting_power.saturating_add(voting_power);

            self.proposals.insert(proposal_id, &proposal);

            self.env().emit_event(VoteCast {
                proposal_id,
                voter: caller,
                support,
                voting_power,
                reason,
            });
            
            self.entered = false;
            Ok(())
        }

        /// Queue a proposal for execution after voting period; starts the timelock countdown
        #[ink(message)]
        pub fn queue_proposal(&mut self, proposal_id: u64) -> Result<()> {
            if self.entered { self.entered = false; return Err(Error::Unauthorized); }
            self.entered = true;

            let proposal = self.proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;

            // Only after voting ends and not executed
            let current_block = self.env().block_number();
            if (current_block as u64) < proposal.voting_end { 
                self.entered = false;
                return Err(Error::ProposalNotExpired); 
            }
            if proposal.executed { 
                self.entered = false;
                return Err(Error::ProposalAlreadyExecuted); 
            }

            // Store queue time
            let now = self.env().block_timestamp();
            self.queue_times.insert(proposal_id, &now);

            let execute_after = now.saturating_add(self.timelock_seconds.saturating_mul(1000));
            self.env().emit_event(ProposalQueued { proposal_id, queued_at: now, execute_after });
            
            self.entered = false;
            Ok(())
        }

        /// Update timelock delay (owner only)
        #[ink(message)]
        pub fn set_timelock_seconds(&mut self, seconds: u64) -> Result<()> {
            if self.env().caller() != self.owner { return Err(Error::Unauthorized); }
            let old = self.timelock_seconds;
            self.timelock_seconds = seconds;
            self.env().emit_event(TimelockUpdated { old_seconds: old, new_seconds: seconds });
            Ok(())
        }

        /// Execute a proposal
        #[ink(message)]
        pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<()> {
            if self.entered { self.entered = false; return Err(Error::Unauthorized); }
            self.entered = true;
            
            let mut proposal = self.proposals.get(proposal_id)
                .ok_or(Error::ProposalNotFound)?;

            // Check if proposal has expired
            let current_block = self.env().block_number();
            if (current_block as u64) < proposal.voting_end { 
                self.entered = false;
                return Err(Error::ProposalNotExpired); 
            }

            // Check if already executed
            if proposal.executed { 
                self.entered = false;
                return Err(Error::ProposalAlreadyExecuted); 
            }

            // Check quorum
            let total_supply = self.get_total_voting_power();
            let quorum_required = total_supply.saturating_mul(self.quorum_percentage as u64).saturating_div(100);
            
            let passed = proposal.yes_votes > proposal.no_votes && proposal.total_voting_power >= quorum_required;
            
            // Require proposal queued and respect timelock if passed
            if passed {
                let queued_at = self.queue_times.get(proposal_id).unwrap_or(0);
                if queued_at == 0 { 
                    self.entered = false;
                    return Err(Error::NotQueued); 
                }
                let now = self.env().block_timestamp();
                let execute_after = queued_at.saturating_add(self.timelock_seconds.saturating_mul(1000));
                if now < execute_after { 
                    self.entered = false;
                    return Err(Error::TimelockNotElapsed); 
                }
            }
            
            // If passed, attempt to execute side effects
            let mut success = passed;
            if passed {
                #[cfg(not(test))]
                {
                    match proposal.proposal_type.clone() {
                        ProposalType::UpdateMinStake(new_min) => {
                            let mut registry = ResourceRegistryRef::from_account_id(self.registry_address);
                            if registry.update_min_stake(new_min).is_err() { success = false; }
                        }
                        ProposalType::UpdateCompensationRate(new_rate) => {
                            let mut grid = GridServiceRef::from_account_id(self.grid_service_address);
                            if grid.update_default_compensation_rate(new_rate).is_err() { success = false; }
                        }
                        ProposalType::UpdateReputationThreshold(threshold) => {
                            let mut registry = ResourceRegistryRef::from_account_id(self.registry_address);
                            if registry.update_reputation_threshold(threshold).is_err() { success = false; }
                        }
                        ProposalType::TreasurySpend(to_bytes, amount) => {
                            let to = ink::primitives::AccountId::from(to_bytes);
                            // Use token transfer from this contract's balance
                            let mut token = PowergridTokenRef::from_account_id(self.token_address);
                            if token.transfer(to, amount, Vec::new()).is_err() { success = false; }
                        }
                        ProposalType::SetTokenMinter(account_bytes, is_minter) => {
                            let account = ink::primitives::AccountId::from(account_bytes);
                            let mut token = PowergridTokenRef::from_account_id(self.token_address);
                            let r = if is_minter { token.add_minter(account) } else { token.remove_minter(account) };
                            if r.is_err() { success = false; }
                        }
                        ProposalType::SetRegistryAuthorizedCaller(account_bytes, is_auth) => {
                            let account = ink::primitives::AccountId::from(account_bytes);
                            let mut registry = ResourceRegistryRef::from_account_id(self.registry_address);
                            let r = if is_auth { registry.add_authorized_caller(account) } else { registry.remove_authorized_caller(account) };
                            if r.is_err() { success = false; }
                        }
                        ProposalType::SetGridAuthorizedCaller(account_bytes, is_auth) => {
                            let account = ink::primitives::AccountId::from(account_bytes);
                            let mut grid = GridServiceRef::from_account_id(self.grid_service_address);
                            let r = if is_auth { grid.add_authorized_caller(account) } else { grid.remove_authorized_caller(account) };
                            if r.is_err() { success = false; }
                        }
                        ProposalType::SystemUpgrade | ProposalType::Other(_) => {
                            success = true;
                        }
                    }
                }
            }

            // Mark executed only on success; if failed, keep it active for potential retry/fix
            if passed && success { proposal.executed = true; proposal.active = false; }
            if passed && !success { proposal.active = true; }
            self.proposals.insert(proposal_id, &proposal);

            self.env().emit_event(ProposalExecuted {
                proposal_id,
                successful: success,
            });
            
            self.entered = false;
            Ok(())
        }

        /// Get proposal details
        #[ink(message)]
        pub fn get_proposal(&self, proposal_id: u64) -> Option<Proposal> {
            self.proposals.get(proposal_id)
        }

        /// Get voting status
        #[ink(message)]
        pub fn has_voted(&self, proposal_id: u64, voter: AccountId) -> bool {
            let voter_bytes = ink_account_to_bytes(voter);
            self.votes.contains((proposal_id, voter_bytes))
        }

        /// Get governance parameters
        #[ink(message)]
        pub fn get_governance_params(&self) -> (Balance, u64, u32) {
            (self.min_voting_power, self.voting_duration_blocks, self.quorum_percentage)
        }

        /// Get voting power from PSP22 token balance
        #[allow(clippy::cast_possible_truncation)]
        fn get_voting_power(&self, account: AccountId) -> u64 {
            let token = PowergridTokenRef::from_account_id(self.token_address);
            let bal: u128 = token.balance_of(account);
            // Downcast safely; governance uses u64 voting units
            bal.min(u128::from(u64::MAX)) as u64
        }

        /// Get total voting power from PSP22 total_supply
        #[allow(clippy::cast_possible_truncation)]
        fn get_total_voting_power(&self) -> u64 {
            let token = PowergridTokenRef::from_account_id(self.token_address);
            let total: u128 = token.total_supply();
            total.min(u128::from(u64::MAX)) as u64
        }
    }
}
