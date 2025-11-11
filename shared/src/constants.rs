/// Token decimals for Substrate (standard)
pub const SUBSTRATE_DECIMALS: u8 = 12;

/// Conversion factor for Substrate native token
pub const SUBSTRATE_UNIT: u128 = 1_000_000_000_000; // 10^12

/// Common stake amounts in native units
pub const ONE_TOKEN: u128 = SUBSTRATE_UNIT;
pub const MIN_STAKE_DEFAULT: u128 = ONE_TOKEN; // 1 token minimum

/// Helper functions for unit conversion
pub fn tokens_to_native(tokens: u128) -> u128 {
    tokens.saturating_mul(SUBSTRATE_UNIT)
}

pub fn native_to_tokens(native: u128) -> u128 {
    native / SUBSTRATE_UNIT
}

