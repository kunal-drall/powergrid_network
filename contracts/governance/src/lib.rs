#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod governance {
    use powergrid_shared::{Proposal, ProposalType};
    use ink::storage::Mapping;
    use ink::prelude::{string::String, vec::Vec};

    #[ink(storage)]
    pub struct Governance {
        // Contract addresses
        token_address: AccountId,
        registry_address: AccountId,
        grid_service_address: AccountId,
        
        // Governance parameters
        admin: AccountId,
        proposal_threshold: Balance, // Minimum tokens to create proposal
        voting_period: u64, // Voting period in milliseconds
        execution_delay: u64, // Delay before execution in milliseconds
        quorum_threshold: u32, // Percentage of total staked tokens (in basis points)
        
        // Proposals
        proposals: Mapping<u64, Proposal>,
        proposal_count: u64,
        
        // Voting tracking
        votes: Mapping<(u64, AccountId), (bool, Balance)>, // (proposal_id, voter) -> (vote, weight)
        voter_counts: Mapping<u64, (u32, u32)>, // proposal_id -> (yes_count, no_count)
        
        // Treasury management
        treasury_balance: Balance,
        treasury_proposals: Mapping<u64, bool>, // proposal_id -> is_treasury_proposal
        
        // Execution tracking
        executed_proposals: Mapping<u64, bool>,
        execution_queue: Mapping<u64, u64>, // proposal_id -> execution_timestamp
    }

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
        vote: bool,
        weight: Balance,
        reason: String,
    }

    #[ink(event)]
    pub struct ProposalExecuted {
        #[ink(topic)]
        proposal_id: u64,
        success: bool,
        result: String,
    }

    #[ink(event)]
    pub struct QuorumReached {
        #[ink(topic)]
        proposal_id: u64,
        total_voting_power: u64,
        quorum_threshold: u64,
    }

    #[ink(event)]
    pub struct TreasuryDeposit {
        #[ink(topic)]
        from: AccountId,
        amount: Balance,
        reason: String,
    }

    impl Governance {
        #[ink(constructor)]
        pub fn new(
            token_address: AccountId,
            registry_address: AccountId,
            grid_service_address: AccountId,
            proposal_threshold: Balance,
            voting_period: u64,
            quorum_threshold: u32,
        ) -> Self {
            Self {
                token_address,
                registry_address,
                grid_service_address,
                admin: Self::env().caller(),
                proposal_threshold,
                voting_period,
                execution_delay: 24 * 60 * 60 * 1000, // 24 hours in milliseconds
                quorum_threshold,
                proposals: Mapping::default(),
                proposal_count: 0,
                votes: Mapping::default(),
                voter_counts: Mapping::default(),
                treasury_balance: 0,
                treasury_proposals: Mapping::default(),
                executed_proposals: Mapping::default(),
                execution_queue: Mapping::default(),
            }
        }

        // ========================================================================
        // PROPOSAL CREATION AND MANAGEMENT
        // ========================================================================

        #[ink(message)]
        pub fn create_proposal(
            &mut self,
            proposal_type: ProposalType,
            description: String,
        ) -> Result<u64, String> {
            let caller = self.env().caller();
            let current_time = self.env().block_timestamp();
            
            // Check if caller has enough staked tokens to create proposal
            let voting_power = self.get_voting_power(caller)?;
            if voting_power < self.proposal_threshold {
                return Err("Insufficient staked tokens to create proposal".into());
            }

            let proposal_id = self.proposal_count;
            let voting_end = current_time + self.voting_period;

            let proposal = Proposal {
                proposer: caller,
                proposal_type: proposal_type.clone(),
                description: description.clone(),
                yes_votes: 0,
                no_votes: 0,
                total_voting_power: 0,
                created_at: current_time,
                voting_end,
                executed: false,
                active: true,
            };

            self.proposals.insert(proposal_id, &proposal);
            self.proposal_count += 1;
            self.voter_counts.insert(proposal_id, &(0, 0));

            // Mark treasury proposals
            if matches!(proposal_type, ProposalType::TreasurySpend(_, _)) {
                self.treasury_proposals.insert(proposal_id, &true);
            }

            self.env().emit_event(ProposalCreated {
                proposal_id,
                proposer: caller,
                proposal_type,
                description,
                voting_end,
            });

            Ok(proposal_id)
        }

        #[ink(message)]
        pub fn vote(
            &mut self,
            proposal_id: u64,
            vote: bool,
            reason: String,
        ) -> Result<(), String> {
            let caller = self.env().caller();
            let current_time = self.env().block_timestamp();

            let mut proposal = self.proposals.get(proposal_id)
                .ok_or("Proposal not found")?;

            // Validate voting conditions
            if !proposal.active {
                return Err("Proposal is not active".into());
            }

            if current_time > proposal.voting_end {
                return Err("Voting period has ended".into());
            }

            // Check if already voted
            if self.votes.contains((proposal_id, caller)) {
                return Err("Already voted on this proposal".into());
            }

            // Get voting power (staked token balance)
            let voting_power = self.get_voting_power(caller)?;
            if voting_power == 0 {
                return Err("No voting power (no staked tokens)".into());
            }

            // Record vote
            self.votes.insert((proposal_id, caller), &(vote, voting_power));

            // Update proposal vote counts
            if vote {
                proposal.yes_votes += voting_power as u64;
            } else {
                proposal.no_votes += voting_power as u64;
            }
            proposal.total_voting_power += voting_power as u64;

            // Update voter counts
            let (yes_count, no_count) = self.voter_counts.get(proposal_id).unwrap_or((0, 0));
            if vote {
                self.voter_counts.insert(proposal_id, &(yes_count + 1, no_count));
            } else {
                self.voter_counts.insert(proposal_id, &(yes_count, no_count + 1));
            }

            self.proposals.insert(proposal_id, &proposal);

            // Check if quorum is reached
            self.check_quorum(proposal_id, &proposal)?;

            self.env().emit_event(VoteCast {
                proposal_id,
                voter: caller,
                vote,
                weight: voting_power,
                reason,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<(), String> {
            let current_time = self.env().block_timestamp();

            let mut proposal = self.proposals.get(proposal_id)
                .ok_or("Proposal not found")?;

            // Validate execution conditions
            if proposal.executed {
                return Err("Proposal already executed".into());
            }

            if current_time <= proposal.voting_end {
                return Err("Voting period has not ended".into());
            }

            // Check if execution delay has passed
            if let Some(execution_time) = self.execution_queue.get(proposal_id) {
                if current_time < execution_time {
                    return Err("Execution delay has not passed".into());
                }
            } else {
                // Set execution time if not set
                let execution_time = current_time + self.execution_delay;
                self.execution_queue.insert(proposal_id, &execution_time);
                return Err("Execution delay period started, try again later".into());
            }

            // Check if proposal passed
            let passed = self.proposal_passed(&proposal)?;
            if !passed {
                proposal.executed = true;
                proposal.active = false;
                self.proposals.insert(proposal_id, &proposal);
                
                self.env().emit_event(ProposalExecuted {
                    proposal_id,
                    success: false,
                    result: "Proposal did not pass".into(),
                });
                
                return Ok(());
            }

            // Execute proposal
            let execution_result = self.execute_proposal_action(&proposal.proposal_type);
            
            proposal.executed = true;
            proposal.active = false;
            self.proposals.insert(proposal_id, &proposal);
            self.executed_proposals.insert(proposal_id, &execution_result.is_ok());

            let success = execution_result.is_ok();
            let result_message = match execution_result {
                Ok(msg) => msg,
                Err(err) => format!("Execution failed: {}", err),
            };

            self.env().emit_event(ProposalExecuted {
                proposal_id,
                success,
                result: result_message,
            });

            Ok(())
        }

        // ========================================================================
        // PROPOSAL EXECUTION LOGIC
        // ========================================================================

        fn execute_proposal_action(&mut self, proposal_type: &ProposalType) -> Result<String, String> {
            match proposal_type {
                ProposalType::UpdateMinStake(new_stake) => {
                    // In a real implementation, this would make a cross-contract call
                    Ok(format!("Min stake updated to {}", new_stake))
                },
                ProposalType::UpdateCompensationRate(new_rate) => {
                    // In a real implementation, this would make a cross-contract call
                    Ok(format!("Compensation rate updated to {}", new_rate))
                },
                ProposalType::UpdateReputationThreshold(new_threshold) => {
                    // In a real implementation, this would make a cross-contract call
                    Ok(format!("Reputation threshold updated to {}", new_threshold))
                },
                ProposalType::TreasurySpend(recipient, amount) => {
                    if self.treasury_balance < *amount {
                        return Err("Insufficient treasury balance".into());
                    }
                    
                    self.treasury_balance -= amount;
                    // In a real implementation, this would transfer tokens
                    Ok(format!("Treasury spent {} to {:?}", amount, recipient))
                },
                ProposalType::SystemUpgrade => {
                    // In a real implementation, this would trigger system upgrade
                    Ok("System upgrade initiated".into())
                },
                ProposalType::Other(description) => {
                    Ok(format!("Custom proposal executed: {}", description))
                },
            }
        }

        // ========================================================================
        // VOTING POWER AND QUORUM CALCULATIONS
        // ========================================================================

        fn get_voting_power(&self, _account: AccountId) -> Result<Balance, String> {
            // In a real implementation, this would make a cross-contract call to token contract
            // For now, return a simulated staked balance
            Ok(1000) // Simplified implementation
        }

        fn get_total_voting_power(&self) -> Result<Balance, String> {
            // In a real implementation, this would get total staked supply from token contract
            Ok(100000) // Simplified implementation
        }

        fn check_quorum(&self, proposal_id: u64, proposal: &Proposal) -> Result<(), String> {
            let total_voting_power = self.get_total_voting_power()?;
            let required_quorum = (total_voting_power as u128 * self.quorum_threshold as u128 / 10000) as Balance;
            
            if proposal.total_voting_power as Balance >= required_quorum {
                self.env().emit_event(QuorumReached {
                    proposal_id,
                    total_voting_power: proposal.total_voting_power,
                    quorum_threshold: required_quorum as u64,
                });
            }
            
            Ok(())
        }

        fn proposal_passed(&self, proposal: &Proposal) -> Result<bool, String> {
            // Check quorum
            let total_voting_power = self.get_total_voting_power()?;
            let required_quorum = (total_voting_power as u128 * self.quorum_threshold as u128 / 10000) as Balance;
            
            if (proposal.total_voting_power as Balance) < required_quorum {
                return Ok(false); // Quorum not reached
            }

            // Simple majority for most proposals
            let passed = proposal.yes_votes > proposal.no_votes;
            
            // Special handling for treasury proposals (require supermajority)
            if let Some(true) = self.treasury_proposals.get(0) { // Simplified check
                let supermajority_threshold = proposal.total_voting_power * 66 / 100; // 66%
                return Ok(proposal.yes_votes > supermajority_threshold);
            }
            
            Ok(passed)
        }

        // ========================================================================
        // TREASURY MANAGEMENT
        // ========================================================================

        #[ink(message, payable)]
        pub fn deposit_to_treasury(&mut self, reason: String) -> Result<(), String> {
            let caller = self.env().caller();
            let amount = self.env().transferred_value();
            
            if amount == 0 {
                return Err("Deposit amount must be greater than 0".into());
            }

            self.treasury_balance += amount;

            self.env().emit_event(TreasuryDeposit {
                from: caller,
                amount,
                reason,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn get_treasury_balance(&self) -> Balance {
            self.treasury_balance
        }

        // ========================================================================
        // PARAMETER UPDATES
        // ========================================================================

        #[ink(message)]
        pub fn update_governance_parameters(
            &mut self,
            proposal_threshold: Option<Balance>,
            voting_period: Option<u64>,
            execution_delay: Option<u64>,
            quorum_threshold: Option<u32>,
        ) -> Result<(), String> {
            if self.env().caller() != self.admin {
                return Err("Only admin can update parameters during bootstrap".into());
            }

            if let Some(threshold) = proposal_threshold {
                self.proposal_threshold = threshold;
            }
            if let Some(period) = voting_period {
                self.voting_period = period;
            }
            if let Some(delay) = execution_delay {
                self.execution_delay = delay;
            }
            if let Some(quorum) = quorum_threshold {
                if quorum > 10000 { // Max 100%
                    return Err("Quorum threshold cannot exceed 100%".into());
                }
                self.quorum_threshold = quorum;
            }

            Ok(())
        }

        #[ink(message)]
        pub fn transfer_admin(&mut self, new_admin: AccountId) -> Result<(), String> {
            if self.env().caller() != self.admin {
                return Err("Only admin can transfer admin rights".into());
            }
            self.admin = new_admin;
            Ok(())
        }

        // ========================================================================
        // DELEGATION SYSTEM (Advanced Feature)
        // ========================================================================

        #[ink(message)]
        pub fn delegate_voting_power(&mut self, _delegate: AccountId) -> Result<(), String> {
            let _caller = self.env().caller();
            
            // In a real implementation, this would:
            // 1. Record delegation in storage
            // 2. Update voting power calculations
            // 3. Emit delegation events
            
            // For now, just return success
            Ok(())
        }

        #[ink(message)]
        pub fn revoke_delegation(&mut self) -> Result<(), String> {
            let _caller = self.env().caller();
            
            // In a real implementation, this would:
            // 1. Remove delegation record
            // 2. Restore direct voting power
            // 3. Emit revocation events
            
            Ok(())
        }

        // ========================================================================
        // VIEW FUNCTIONS
        // ========================================================================

        #[ink(message)]
        pub fn get_proposal(&self, proposal_id: u64) -> Option<Proposal> {
            self.proposals.get(proposal_id)
        }

        #[ink(message)]
        pub fn get_vote(&self, proposal_id: u64, voter: AccountId) -> Option<(bool, Balance)> {
            self.votes.get((proposal_id, voter))
        }

        #[ink(message)]
        pub fn get_proposal_count(&self) -> u64 {
            self.proposal_count
        }

        #[ink(message)]
        pub fn get_voter_counts(&self, proposal_id: u64) -> (u32, u32) {
            self.voter_counts.get(proposal_id).unwrap_or((0, 0))
        }

        #[ink(message)]
        pub fn is_proposal_executed(&self, proposal_id: u64) -> bool {
            self.executed_proposals.get(proposal_id).unwrap_or(false)
        }

        #[ink(message)]
        pub fn get_governance_parameters(&self) -> (Balance, u64, u64, u32) {
            (
                self.proposal_threshold,
                self.voting_period,
                self.execution_delay,
                self.quorum_threshold,
            )
        }

        #[ink(message)]
        pub fn get_active_proposals(&self) -> Vec<u64> {
            // In a real implementation, this would filter active proposals
            // For now, return empty vector
            Vec::new()
        }

        #[ink(message)]
        pub fn get_executable_proposals(&self) -> Vec<u64> {
            // In a real implementation, this would filter executable proposals
            // For now, return empty vector
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ink::env::test::{default_accounts, set_caller, set_value_transferred};
    use powergrid_shared::ProposalType;

    #[ink::test]
    fn test_proposal_creation() {
        let accounts = default_accounts();
        let mut governance = governance::Governance::new(
            accounts.alice,
            accounts.bob,
            accounts.charlie,
            10000,     // 10k tokens needed to propose
            86400000,  // 24 hours voting period
            2500,      // 25% quorum
        );

        set_caller(accounts.alice);
        
        let result = governance.create_proposal(
            ProposalType::UpdateMinStake(2000),
            "Increase minimum stake to 2000 tokens".into(),
        );

        assert!(result.is_ok());
        let proposal_id = result.unwrap();

        let proposal = governance.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.proposer, accounts.alice);
        assert!(matches!(proposal.proposal_type, ProposalType::UpdateMinStake(2000)));
        assert!(proposal.active);
        assert!(!proposal.executed);
    }

    #[ink::test]
    fn test_voting_on_proposal() {
        let accounts = default_accounts();
        let mut governance = governance::Governance::new(
            accounts.alice,
            accounts.bob,
            accounts.charlie,
            10000,
            86400000,
            2500,
        );

        // Create proposal
        set_caller(accounts.alice);
        let proposal_id = governance.create_proposal(
            ProposalType::UpdateMinStake(2000),
            "Test proposal".into(),
        ).unwrap();

        // Vote on proposal
        let result = governance.vote(proposal_id, true, "I support this".into());
        assert!(result.is_ok());

        let vote = governance.get_vote(proposal_id, accounts.alice).unwrap();
        assert_eq!(vote.0, true); // Vote value
        assert!(vote.1 > 0); // Voting weight

        let proposal = governance.get_proposal(proposal_id).unwrap();
        assert!(proposal.yes_votes > 0);
    }

    #[ink::test]
    fn test_treasury_deposit() {
        let accounts = default_accounts();
        let mut governance = governance::Governance::new(
            accounts.alice,
            accounts.bob,
            accounts.charlie,
            10000,
            86400000,
            2500,
        );

        set_caller(accounts.alice);
        set_value_transferred(50000);

        let result = governance.deposit_to_treasury("Initial funding".into());
        assert!(result.is_ok());

        assert_eq!(governance.get_treasury_balance(), 50000);
    }
}