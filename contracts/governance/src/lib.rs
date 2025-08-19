#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod governance {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;
use powergrid_shared::{Proposal, ProposalType, ink_account_to_bytes};

    /// The Governance contract
    #[ink(storage)]
    pub struct Governance {
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
        votes: Mapping<(u64, [u8; 32]), bool>,
        /// Next proposal ID
        next_proposal_id: u64,
        /// Minimum voting power required to create proposals
        min_voting_power: Balance,
        /// Voting duration in blocks
        voting_duration_blocks: u64,
        /// Quorum percentage (out of 100)
        quorum_percentage: u32,
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
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Governance {
        /// Constructor
        #[ink(constructor)]
        pub fn new(
            token_address: AccountId,
            registry_address: AccountId,
            grid_service_address: AccountId,
            min_voting_power: Balance,
            voting_duration_blocks: u64,
            quorum_percentage: u32,
        ) -> Self {
            Self {
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

            // Check voting power (simplified)
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
                voting_end: voting_end as u64,
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
                voting_end: voting_end as u64,
            });

            Ok(proposal_id)
        }

        /// Vote on a proposal
        #[ink(message)]
        pub fn vote(&mut self, proposal_id: u64, support: bool, reason: String) -> Result<()> {
            let caller = self.env().caller();
            let caller_bytes = ink_account_to_bytes(caller);

            let mut proposal = self.proposals.get(proposal_id)
                .ok_or(Error::ProposalNotFound)?;

            // Check if proposal is still active
            let current_block = self.env().block_number();
            if (current_block as u64) > proposal.voting_end {
                return Err(Error::ProposalExpired);
            }

            // Check if already voted
            if self.votes.contains((proposal_id, caller_bytes)) {
                return Err(Error::AlreadyVoted);
            }

            // Get voting power (simplified)
            let voting_power = self.get_voting_power(caller);
            if voting_power == 0 {
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

            Ok(())
        }

        /// Execute a proposal
        #[ink(message)]
        pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<()> {
            let mut proposal = self.proposals.get(proposal_id)
                .ok_or(Error::ProposalNotFound)?;

            // Check if proposal has expired
            let current_block = self.env().block_number();
            if (current_block as u64) <= proposal.voting_end {
                return Err(Error::ProposalNotExpired);
            }

            // Check if already executed
            if proposal.executed {
                return Err(Error::ProposalAlreadyExecuted);
            }

            // Check quorum
            let total_supply = self.get_total_voting_power();
            let quorum_required = total_supply.saturating_mul(self.quorum_percentage as u64).saturating_div(100);
            
            let passed = proposal.yes_votes > proposal.no_votes && proposal.total_voting_power >= quorum_required;
            
            proposal.executed = true;
            proposal.active = false;
            self.proposals.insert(proposal_id, &proposal);

            self.env().emit_event(ProposalExecuted {
                proposal_id,
                successful: passed,
            });

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

        /// Get voting power (simplified implementation)
        fn get_voting_power(&self, _account: AccountId) -> u64 {
            100 // Simplified: everyone has 100 voting power
        }

        /// Get total voting power (simplified implementation)
        fn get_total_voting_power(&self) -> u64 {
            10000 // Simplified: total voting power is 10000
        }
    }
}
