use ink::prelude::string::String;
use ink::storage::traits::StorageLayout;
use scale::{Decode, Encode};
use scale_info::TypeInfo;

// Make these type aliases public
pub type AccountId = <ink::env::DefaultEnvironment as ink::env::Environment>::AccountId;
pub type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;
pub type Timestamp = u64;

#[derive(Decode, Encode, Clone, TypeInfo, StorageLayout, PartialEq, Debug)]
pub enum DeviceType {
    SmartPlug,
    EV,
    WaterHeater,
    AirConditioner,
    SolarPanel,
    Battery,
    Other(String),
}

#[derive(Decode, Encode, Clone, TypeInfo, StorageLayout, Debug)]
pub struct DeviceMetadata {
    pub device_type: DeviceType,
    pub capacity_watts: u64,
    pub location: String,
    pub manufacturer: String,
    pub model: String,
    pub firmware_version: String,
    pub installation_date: Timestamp,
}

#[derive(Decode, Encode, Clone, TypeInfo, StorageLayout, Debug)]
pub struct Device {
    pub metadata: DeviceMetadata,
    pub stake: Balance,
    pub reputation: u32,
    pub total_energy_contributed: u64,
    pub successful_events: u32,
    pub failed_events: u32,
    pub last_activity: Timestamp,
    pub active: bool,
}

#[derive(Decode, Encode, Clone, TypeInfo, StorageLayout, Debug)]
pub enum GridEventType {
    DemandResponse,
    FrequencyRegulation,
    PeakShaving,
    LoadBalancing,
    Emergency,
}

#[derive(Decode, Encode, Clone, TypeInfo, StorageLayout, Debug)]
pub struct GridEvent {
    pub event_type: GridEventType,
    pub duration_minutes: u64,
    pub base_compensation_rate: Balance,
    pub target_reduction_kw: u64,
    pub created_at: Timestamp,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub active: bool,
    pub total_participants: u32,
    pub total_energy_reduced: u64,
    pub completed: bool,
}

#[derive(Decode, Encode, Clone, TypeInfo, StorageLayout, Debug)]
pub struct Participation {
    pub participant: AccountId,
    pub energy_contributed_wh: u64,
    pub participation_start: Timestamp,
    pub participation_end: Timestamp,
    pub reward_earned: Balance,
    pub verified: bool,
}

#[derive(Decode, Encode, Clone, TypeInfo, StorageLayout, Debug)]
pub enum ProposalType {
    UpdateMinStake(Balance),
    UpdateCompensationRate(Balance),
    UpdateReputationThreshold(u32),
    TreasurySpend(AccountId, Balance),
    SystemUpgrade,
    Other(String),
}

#[derive(Decode, Encode, Clone, TypeInfo, StorageLayout, Debug)]
pub struct Proposal {
    pub proposer: AccountId,
    pub proposal_type: ProposalType,
    pub description: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub total_voting_power: u64,
    pub created_at: Timestamp,
    pub voting_end: Timestamp,
    pub executed: bool,
    pub active: bool,
}