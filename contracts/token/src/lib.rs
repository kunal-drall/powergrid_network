#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod powergrid_token {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;
use powergrid_shared::ink_account_to_bytes;

    /// The PowergridToken contract
    #[ink(storage)]
    pub struct PowergridToken {
        /// Total token supply
        total_supply: Balance,
        /// Balances of each account
        balances: Mapping<[u8; 32], Balance>,
        /// Allowances mapping (owner -> spender -> amount)
        allowances: Mapping<([u8; 32], [u8; 32]), Balance>,
        /// Token metadata
        name: String,
        symbol: String,
        decimals: u8,
        /// Contract owner
        owner: AccountId,
        /// Authorized minters
        minters: Vec<AccountId>,
        /// Authorized burners  
        burners: Vec<AccountId>,
    }

    /// Events emitted by the contract
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    #[ink(event)]
    pub struct Mint {
        #[ink(topic)]
        to: AccountId,
        value: Balance,
    }

    #[ink(event)]
    pub struct Burn {
        #[ink(topic)]
        from: AccountId,
        value: Balance,
    }

    #[ink(event)]
    pub struct MinterAdded {
        #[ink(topic)]
        minter: AccountId,
    }

    #[ink(event)]
    pub struct MinterRemoved {
        #[ink(topic)]
        minter: AccountId,
    }

    /// Errors
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance,
        Unauthorized,
        InvalidAmount,
        SelfApproval,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl PowergridToken {
        /// Constructor
        #[ink(constructor)]
        pub fn new(
            name: String,
            symbol: String,
            decimals: u8,
            initial_supply: Balance,
        ) -> Self {
            let caller = Self::env().caller();
            let caller_bytes = ink_account_to_bytes(caller);
            let mut balances = Mapping::default();
            balances.insert(caller_bytes, &initial_supply);

            let instance = Self {
                total_supply: initial_supply,
                balances,
                allowances: Mapping::default(),
                name,
                symbol,
                decimals,
                owner: caller,
                minters: Vec::new(),
                burners: Vec::new(),
            };

            // Emit initial transfer event
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            });

            instance
        }

        /// Get token name
        #[ink(message)]
        pub fn name(&self) -> String {
            self.name.clone()
        }

        /// Get token symbol
        #[ink(message)]
        pub fn symbol(&self) -> String {
            self.symbol.clone()
        }

        /// Get token decimals
        #[ink(message)]
        pub fn decimals(&self) -> u8 {
            self.decimals
        }

        /// Get total supply
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /// Get balance of an account
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            let owner_bytes = ink_account_to_bytes(owner);
            self.balances.get(owner_bytes).unwrap_or(0)
        }

        /// Transfer tokens
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(from, to, value)?;
            Ok(())
        }

        /// Approve spender
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            
            if owner == spender {
                return Err(Error::SelfApproval);
            }

            let owner_bytes = ink_account_to_bytes(owner);
            let spender_bytes = ink_account_to_bytes(spender);
            
            self.allowances.insert((owner_bytes, spender_bytes), &value);

            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });

            Ok(())
        }

        /// Get allowance
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            let owner_bytes = ink_account_to_bytes(owner);
            let spender_bytes = ink_account_to_bytes(spender);
            self.allowances.get((owner_bytes, spender_bytes)).unwrap_or(0)
        }

        /// Transfer from account
        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            let from_bytes = ink_account_to_bytes(from);
            let caller_bytes = ink_account_to_bytes(caller);

            if caller != from {
                let current_allowance = self.allowances.get((from_bytes, caller_bytes)).unwrap_or(0);
                
                if current_allowance < value {
                    return Err(Error::InsufficientAllowance);
                }

                self.allowances.insert((from_bytes, caller_bytes), &current_allowance.saturating_sub(value));
            }

            self.transfer_from_to(from, to, value)?;
            Ok(())
        }

        /// Mint tokens (authorized minters only)
        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            
            if caller != self.owner && !self.minters.contains(&caller) {
                return Err(Error::Unauthorized);
            }

            if value == 0 {
                return Err(Error::InvalidAmount);
            }

            let to_bytes = ink_account_to_bytes(to);
            let balance = self.balances.get(to_bytes).unwrap_or(0);
            self.balances.insert(to_bytes, &balance.saturating_add(value));
            self.total_supply = self.total_supply.saturating_add(value);

            self.env().emit_event(Transfer {
                from: None,
                to: Some(to),
                value,
            });

            self.env().emit_event(Mint {
                to,
                value,
            });

            Ok(())
        }

        /// Burn tokens (authorized burners only)
        #[ink(message)]
        pub fn burn(&mut self, from: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            
            if caller != self.owner && !self.burners.contains(&caller) && caller != from {
                return Err(Error::Unauthorized);
            }

            if value == 0 {
                return Err(Error::InvalidAmount);
            }

            let from_bytes = ink_account_to_bytes(from);
            let balance = self.balances.get(from_bytes).unwrap_or(0);
            
            if balance < value {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(from_bytes, &balance.saturating_sub(value));
            self.total_supply = self.total_supply.saturating_sub(value);

            self.env().emit_event(Transfer {
                from: Some(from),
                to: None,
                value,
            });

            self.env().emit_event(Burn {
                from,
                value,
            });

            Ok(())
        }

        /// Add minter (owner only)
        #[ink(message)]
        pub fn add_minter(&mut self, minter: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            if !self.minters.contains(&minter) {
                self.minters.push(minter);
                
                self.env().emit_event(MinterAdded {
                    minter,
                });
            }

            Ok(())
        }

        /// Remove minter (owner only)
        #[ink(message)]
        pub fn remove_minter(&mut self, minter: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            self.minters.retain(|&x| x != minter);
            
            self.env().emit_event(MinterRemoved {
                minter,
            });

            Ok(())
        }

        /// Add burner (owner only)
        #[ink(message)]
        pub fn add_burner(&mut self, burner: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            if !self.burners.contains(&burner) {
                self.burners.push(burner);
            }

            Ok(())
        }

        /// Remove burner (owner only)
        #[ink(message)]
        pub fn remove_burner(&mut self, burner: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            self.burners.retain(|&x| x != burner);
            Ok(())
        }

        /// Get minters (owner only)
        #[ink(message)]
        pub fn get_minters(&self) -> Result<Vec<AccountId>> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            Ok(self.minters.clone())
        }

        /// Get burners (owner only)
        #[ink(message)]
        pub fn get_burners(&self) -> Result<Vec<AccountId>> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            Ok(self.burners.clone())
        }

        /// Check if account is minter
        #[ink(message)]
        pub fn is_minter(&self, account: AccountId) -> bool {
            account == self.owner || self.minters.contains(&account)
        }

        /// Check if account is burner
        #[ink(message)]
        pub fn is_burner(&self, account: AccountId) -> bool {
            account == self.owner || self.burners.contains(&account)
        }

        /// Get owner
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Transfer ownership (owner only)
        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            self.owner = new_owner;
            Ok(())
        }

        /// Internal transfer function
        fn transfer_from_to(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            if value == 0 {
                return Err(Error::InvalidAmount);
            }

            let from_bytes = ink_account_to_bytes(from);
            let to_bytes = ink_account_to_bytes(to);

            let from_balance = self.balances.get(from_bytes).unwrap_or(0);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(from_bytes, &from_balance.saturating_sub(value));
            
            let to_balance = self.balances.get(to_bytes).unwrap_or(0);
            self.balances.insert(to_bytes, &to_balance.saturating_add(value));

            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });

            Ok(())
        }

        /// Batch transfer (gas efficient for multiple transfers)
        #[ink(message)]
        pub fn batch_transfer(&mut self, recipients: Vec<(AccountId, Balance)>) -> Result<()> {
            let from = self.env().caller();
            
            for (to, value) in recipients {
                self.transfer_from_to(from, to, value)?;
            }
            
            Ok(())
        }

        /// Get token info
        #[ink(message)]
        pub fn token_info(&self) -> (String, String, u8, Balance) {
            (self.name.clone(), self.symbol.clone(), self.decimals, self.total_supply)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test::{default_accounts, set_caller, DefaultAccounts};
        use ink::env::DefaultEnvironment;

        #[ink::test]
        fn test_token_creation() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let token = PowergridToken::new(
                "PowerGrid Token".into(),
                "PGT".into(),
                18,
                1_000_000_000_000_000_000_000, // 1000 tokens with 18 decimals
            );

            assert_eq!(token.name(), "PowerGrid Token");
            assert_eq!(token.symbol(), "PGT");
            assert_eq!(token.decimals(), 18);
            assert_eq!(token.total_supply(), 1_000_000_000_000_000_000_000);
            assert_eq!(token.balance_of(accounts.alice), 1_000_000_000_000_000_000_000);
        }

        #[ink::test]
        fn test_transfer() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut token = PowergridToken::new(
                "Test Token".into(),
                "TEST".into(),
                18,
                1000,
            );

            // Transfer from Alice to Bob
            let result = token.transfer(accounts.bob, 100);
            assert!(result.is_ok());

            assert_eq!(token.balance_of(accounts.alice), 900);
            assert_eq!(token.balance_of(accounts.bob), 100);
        }

        #[ink::test]
        fn test_approve_and_transfer_from() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut token = PowergridToken::new("Test".into(), "TEST".into(), 18, 1000);

            // Alice approves Bob to spend 100 tokens
            let result = token.approve(accounts.bob, 100);
            assert!(result.is_ok());
            assert_eq!(token.allowance(accounts.alice, accounts.bob), 100);

            // Bob transfers from Alice to Charlie
            set_caller::<DefaultEnvironment>(accounts.bob);
            let result = token.transfer_from(accounts.alice, accounts.charlie, 50);
            assert!(result.is_ok());

            assert_eq!(token.balance_of(accounts.alice), 950);
            assert_eq!(token.balance_of(accounts.charlie), 50);
            assert_eq!(token.allowance(accounts.alice, accounts.bob), 50);
        }

        #[ink::test]
        fn test_mint() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut token = PowergridToken::new("Test".into(), "TEST".into(), 18, 1000);

            // Mint as owner
            let result = token.mint(accounts.bob, 500);
            assert!(result.is_ok());

            assert_eq!(token.balance_of(accounts.bob), 500);
            assert_eq!(token.total_supply(), 1500);
        }

        #[ink::test]
        fn test_burn() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut token = PowergridToken::new("Test".into(), "TEST".into(), 18, 1000);

            // Burn own tokens
            let result = token.burn(accounts.alice, 200);
            assert!(result.is_ok());

            assert_eq!(token.balance_of(accounts.alice), 800);
            assert_eq!(token.total_supply(), 800);
        }

        #[ink::test]
        fn test_minter_management() {
            let accounts: DefaultAccounts<DefaultEnvironment> = default_accounts();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut token = PowergridToken::new("Test".into(), "TEST".into(), 18, 1000);

            // Add minter
            let result = token.add_minter(accounts.bob);
            assert!(result.is_ok());
            assert!(token.is_minter(accounts.bob));

            // Bob can now mint
            set_caller::<DefaultEnvironment>(accounts.bob);
            let result = token.mint(accounts.charlie, 100);
            assert!(result.is_ok());

            assert_eq!(token.balance_of(accounts.charlie), 100);
        }
    }
}