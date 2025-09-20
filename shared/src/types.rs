use ink::prelude::string::String;
use scale::{Decode, Encode};
use scale_info::TypeInfo;

#[cfg(feature = "std")]
use ink::storage::traits::StorageLayout;

// Use [u8; 32] directly - no type alias to avoid confusion
pub type Balance = u128;
pub type Timestamp = u64;

// Helper functions for AccountId conversion
pub fn ink_account_to_bytes(account: ink::primitives::AccountId) -> [u8; 32] {
    let bytes: &[u8] = account.as_ref();
    bytes.try_into().unwrap_or([0u8; 32])
}

pub fn bytes_to_ink_account(bytes: [u8; 32]) -> ink::primitives::AccountId {
    ink::primitives::AccountId::from(bytes)
}

#[derive(Decode, Encode, Clone, TypeInfo, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
#[repr(u8)]
pub enum DeviceType {
    SmartPlug = 0,
    EV = 1,
    WaterHeater = 2,
    AirConditioner = 3,
    SolarPanel = 4,
    Battery = 5,
    Other(String) = 6,
}

#[derive(Decode, Encode, Clone, TypeInfo, Debug)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
pub struct DeviceMetadata {
    pub device_type: DeviceType,
    pub capacity_watts: u64,
    pub location: String,
    pub manufacturer: String,
    pub model: String,
    pub firmware_version: String,
    pub installation_date: Timestamp,
}

#[derive(Decode, Encode, Clone, TypeInfo, Debug)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
pub struct Device {
    pub metadata: DeviceMetadata,
    pub stake: Balance,
    pub reputation: u32,
    pub total_energy_contributed: u64,
    pub successful_events: u32,
    pub failed_events: u32,
    pub last_activity: Timestamp,
    pub active: bool,
    pub version: u32,
    pub last_updated: Timestamp,
}

#[derive(Decode, Encode, Clone, TypeInfo, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
#[repr(u8)]
pub enum GridEventType {
    DemandResponse = 0,
    FrequencyRegulation = 1,
    PeakShaving = 2,
    LoadBalancing = 3,
    Emergency = 4,
}

#[derive(Decode, Encode, Clone, TypeInfo, Debug)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
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

#[derive(Decode, Encode, Clone, TypeInfo, Debug)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
pub struct GridSignal {
    pub event_type: GridEventType,
    pub duration_minutes: u64,
    pub target_reduction_kw: u64,
    /// Severity scale 1-5 used to scale compensation rate
    pub severity: u8,
    /// If true, create/start an event with the given parameters
    pub start: bool,
    /// If present, attempt to complete this event
    pub complete_event_id: Option<u64>,
}

#[derive(Decode, Encode, Clone, TypeInfo, Debug)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
pub struct Participation {
    pub participant: [u8; 32],
    pub energy_contributed_wh: u64,
    pub participation_start: Timestamp,
    pub participation_end: Timestamp,
    pub reward_earned: Balance,
    pub verified: bool,
    pub paid: bool,
}

#[derive(Decode, Encode, Clone, TypeInfo, Debug)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
#[repr(u8)]
pub enum ProposalType {
    UpdateMinStake(Balance) = 0,
    UpdateCompensationRate(Balance) = 1,
    UpdateReputationThreshold(u32) = 2,
    TreasurySpend([u8; 32], Balance) = 3,
    SystemUpgrade = 4,
    Other(String) = 5,
    /// Governance role management
    SetTokenMinter([u8; 32], bool) = 6,
    SetRegistryAuthorizedCaller([u8; 32], bool) = 7,
    SetGridAuthorizedCaller([u8; 32], bool) = 8,
}

#[derive(Decode, Encode, Clone, TypeInfo, Debug)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
pub struct Proposal {
    pub proposer: [u8; 32],
    pub proposal_type: ProposalType,
    pub description: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub total_voting_power: u64,
    pub created_at: Timestamp,
    pub voting_end: u64,
    pub executed: bool,
    pub active: bool,
}
