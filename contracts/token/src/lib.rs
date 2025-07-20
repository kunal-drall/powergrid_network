#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod powergrid_token {
    use powergrid_shared::TokenInterface;
    use ink::storage::Mapping;
    use ink::prelude::{string::String, vec::Vec};
use ink::primitives::AccountId as InkAccountId;

    #[ink(storage)]
    pub struct PowergridToken {
        // Basic token data
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
        allowances: Mapping<(AccountId, AccountId), Balance>,
        
        // Token metadata
        name: String,
        symbol: String,
        decimals: u8,
        
        // Access control
        admin: AccountId,
        governance_address: Option<AccountId>,
        authorized_minters: Mapping<AccountId, bool>,
        authorized_burners: Mapping<AccountId, bool>,
        
        // Advanced features
        frozen_accounts: Mapping<AccountId, bool>,
        mintable: bool,
        max_supply: Balance,
        
        // Staking for governance
        staked_balances: Mapping<AccountId, Balance>,
        total_staked: Balance,
        
        // Reward distribution tracking
        total_rewards_distributed: Balance,
        reward_pools: Mapping<String, Balance>, // pool_name -> balance
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: AccountId,
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
        reason: String,
    }

    #[ink(event)]
    pub struct Burn {
        #[ink(topic)]
        from: AccountId,
        value: Balance,
        reason: String,
    }

    #[ink(event)]
    pub struct Stake {
        #[ink(topic)]
        account: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct Unstake {
        #[ink(topic)]
        account: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct RewardDistribution {
        #[ink(topic)]
        pool: String,
        total_amount: Balance,
        recipient_count: u32,
    }

    impl PowergridToken {
        #[ink(constructor)]
        pub fn new(
            initial_supply: Balance,
            name: String,
            symbol: String,
            decimals: u8,
            max_supply: Balance,
        ) -> Self {
            let caller = Self::env().caller();
            let mut balances = Mapping::default();
            balances.insert(caller, &initial_supply);
            
            Self::env().emit_event(Transfer {
                from: None,
                to: caller,
                value: initial_supply,
            });

            Self {
                total_supply: initial_supply,
                balances,
                allowances: Mapping::default(),
                name,
                symbol,
                decimals,
                admin: caller,
                governance_address: None,
                authorized_minters: Mapping::default(),
                authorized_burners: Mapping::default(),
                frozen_accounts: Mapping::default(),
                mintable: true,
                max_supply,
                staked_balances: Mapping::default(),
                total_staked: 0,
                total_rewards_distributed: 0,
                reward_pools: Mapping::default(),
            }
        }

        // ========================================================================
        // BASIC ERC20 FUNCTIONALITY
        // ========================================================================

        #[ink(message)]
        pub fn name(&self) -> String {
            self.name.clone()
        }

        #[ink(message)]
        pub fn symbol(&self) -> String {
            self.symbol.clone()
        }

        #[ink(message)]
        pub fn decimals(&self) -> u8 {
            self.decimals
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap_or(0)
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or(0)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<(), String> {
            let from = self.env().caller();
            self.transfer_from_to(from, to, value)
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), String> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), &value);
            
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            
            Ok(())
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<(), String> {
            let caller = self.env().caller();
            
            if caller != from {
                let allowance = self.allowance(from, caller);
                if allowance < value {
                    return Err("Insufficient allowance".into());
                }
                self.allowances.insert((from, caller), &(allowance - value));
            }
            
            self.transfer_from_to(from, to, value)
        }

        fn transfer_from_to(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<(), String> {
            // Check if accounts are frozen
            if self.frozen_accounts.get(from).unwrap_or(false) {
                return Err("From account is frozen".into());
            }
            if self.frozen_accounts.get(to).unwrap_or(false) {
                return Err("To account is frozen".into());
            }

            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err("Insufficient balance".into());
            }

            self.balances.insert(from, &(from_balance - value));
            let to_balance = self.balance_of(to);
            self.balances.insert(to, &(to_balance + value));

            self.env().emit_event(Transfer {
                from: Some(from),
                to,
                value,
            });

            Ok(())
        }

        // ========================================================================
        // MINTING AND BURNING
        // ========================================================================

        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, value: Balance, reason: String) -> Result<(), String> {
            self.ensure_can_mint()?;

            if self.total_supply + value > self.max_supply {
                return Err("Would exceed max supply".into());
            }

            if self.frozen_accounts.get(to).unwrap_or(false) {
                return Err("Target account is frozen".into());
            }

            self.total_supply += value;
            let balance = self.balance_of(to);
            self.balances.insert(to, &(balance + value));

            self.env().emit_event(Transfer {
                from: None,
                to,
                value,
            });

            self.env().emit_event(Mint {
                to,
                value,
                reason,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn burn(&mut self, from: AccountId, value: Balance, reason: String) -> Result<(), String> {
            self.ensure_can_burn()?;

            let balance = self.balance_of(from);
            if balance < value {
                return Err("Insufficient balance to burn".into());
            }

            self.balances.insert(from, &(balance - value));
            self.total_supply -= value;

            self.env().emit_event(Transfer {
                from: Some(from),
                to: AccountId::from([0u8; 32]),
                value,
            });

            self.env().emit_event(Burn {
                from,
                value,
                reason,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn batch_mint(&mut self, recipients: Vec<(AccountId, Balance)>, reason: String) -> Result<(), String> {
            self.ensure_can_mint()?;

            let total_mint: Balance = recipients.iter().map(|(_, amount)| *amount).sum();
            
            if self.total_supply + total_mint > self.max_supply {
                return Err("Batch mint would exceed max supply".into());
            }

            for (to, value) in recipients {
                if self.frozen_accounts.get(to).unwrap_or(false) {
                    continue; // Skip frozen accounts
                }

                let balance = self.balance_of(to);
                self.balances.insert(to, &(balance + value));

                self.env().emit_event(Transfer {
                    from: None,
                    to,
                    value,
                });

                self.env().emit_event(Mint {
                    to,
                    value,
                    reason: reason.clone(),
                });
            }

            self.total_supply += total_mint;
            Ok(())
        }

        // ========================================================================
        // STAKING FOR GOVERNANCE
        // ========================================================================

        #[ink(message)]
        pub fn stake(&mut self, amount: Balance) -> Result<(), String> {
            let caller = self.env().caller();
            let balance = self.balance_of(caller);
            
            if balance < amount {
                return Err("Insufficient balance to stake".into());
            }

            if self.frozen_accounts.get(caller).unwrap_or(false) {
                return Err("Account is frozen".into());
            }

            // Transfer tokens from balance to staked balance
            self.balances.insert(caller, &(balance - amount));
            let staked = self.staked_balances.get(caller).unwrap_or(0);
            self.staked_balances.insert(caller, &(staked + amount));
            self.total_staked += amount;

            self.env().emit_event(Stake {
                account: caller,
                amount,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn unstake(&mut self, amount: Balance) -> Result<(), String> {
            let caller = self.env().caller();
            let staked = self.staked_balances.get(caller).unwrap_or(0);
            
            if staked < amount {
                return Err("Insufficient staked balance".into());
            }

            if self.frozen_accounts.get(caller).unwrap_or(false) {
                return Err("Account is frozen".into());
            }

            // Transfer tokens from staked balance back to regular balance
            self.staked_balances.insert(caller, &(staked - amount));
            let balance = self.balance_of(caller);
            self.balances.insert(caller, &(balance + amount));
            self.total_staked -= amount;

            self.env().emit_event(Unstake {
                account: caller,
                amount,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn staked_balance_of(&self, account: AccountId) -> Balance {
            self.staked_balances.get(account).unwrap_or(0)
        }

        #[ink(message)]
        pub fn total_staked_supply(&self) -> Balance {
            self.total_staked
        }

        // ========================================================================
        // REWARD DISTRIBUTION SYSTEM
        // ========================================================================

        #[ink(message)]
        pub fn create_reward_pool(&mut self, pool_name: String, initial_amount: Balance) -> Result<(), String> {
            self.ensure_admin_or_governance()?;

            if self.reward_pools.contains(&pool_name) {
                return Err("Reward pool already exists".into());
            }

            // Mint tokens for the reward pool
            self.mint(self.admin, initial_amount, format!("Reward pool creation: {}", pool_name))?;
            self.reward_pools.insert(&pool_name, &initial_amount);

            Ok(())
        }

        #[ink(message)]
        pub fn distribute_rewards(
            &mut self,
            pool_name: String,
            recipients: Vec<(AccountId, Balance)>,
        ) -> Result<(), String> {
            self.ensure_can_mint()?;

            let total_distribution: Balance = recipients.iter().map(|(_, amount)| *amount).sum();
            let pool_balance = self.reward_pools.get(&pool_name).unwrap_or(0);

            if pool_balance < total_distribution {
                return Err("Insufficient pool balance".into());
            }

            // Distribute rewards by minting to recipients
            for (recipient, amount) in &recipients {
                self.mint(*recipient, *amount, format!("Reward from pool: {}", pool_name))?;
            }

            // Update pool balance
            self.reward_pools.insert(&pool_name, &(pool_balance - total_distribution));
            self.total_rewards_distributed += total_distribution;

            self.env().emit_event(RewardDistribution {
                pool: pool_name,
                total_amount: total_distribution,
                recipient_count: recipients.len() as u32,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn get_reward_pool_balance(&self, pool_name: String) -> Balance {
            self.reward_pools.get(&pool_name).unwrap_or(0)
        }

        // ========================================================================
        // ACCESS CONTROL AND ADMIN FUNCTIONS
        // ========================================================================

        #[ink(message)]
        pub fn authorize_minter(&mut self, minter: AccountId) -> Result<(), String> {
            self.ensure_admin_or_governance()?;
            self.authorized_minters.insert(minter, &true);
            Ok(())
        }

        #[ink(message)]
        pub fn revoke_minter(&mut self, minter: AccountId) -> Result<(), String> {
            self.ensure_admin_or_governance()?;
            self.authorized_minters.insert(minter, &false);
            Ok(())
        }

        #[ink(message)]
        pub fn authorize_burner(&mut self, burner: AccountId) -> Result<(), String> {
            self.ensure_admin_or_governance()?;
            self.authorized_burners.insert(burner, &true);
            Ok(())
        }

        #[ink(message)]
        pub fn freeze_account(&mut self, account: AccountId) -> Result<(), String> {
            self.ensure_admin_or_governance()?;
            self.frozen_accounts.insert(account, &true);
            Ok(())
        }

        #[ink(message)]
        pub fn unfreeze_account(&mut self, account: AccountId) -> Result<(), String> {
            self.ensure_admin_or_governance()?;
            self.frozen_accounts.insert(account, &false);
            Ok(())
        }

        #[ink(message)]
        pub fn set_governance_address(&mut self, governance_address: AccountId) -> Result<(), String> {
            if self.env().caller() != self.admin {
                return Err("Only admin can set governance address".into());
            }
            self.governance_address = Some(governance_address);
            Ok(())
        }

        #[ink(message)]
        pub fn set_mintable(&mut self, mintable: bool) -> Result<(), String> {
            self.ensure_admin_or_governance()?;
            self.mintable = mintable;
            Ok(())
        }

        #[ink(message)]
        pub fn update_max_supply(&mut self, new_max_supply: Balance) -> Result<(), String> {
            self.ensure_admin_or_governance()?;
            
            if new_max_supply < self.total_supply {
                return Err("New max supply cannot be less than current supply".into());
            }
            
            self.max_supply = new_max_supply;
            Ok(())
        }

        fn ensure_can_mint(&self) -> Result<(), String> {
            if !self.mintable {
                return Err("Minting is disabled".into());
            }

            let caller = self.env().caller();
            if caller == self.admin || 
               self.governance_address == Some(caller) ||
               self.authorized_minters.get(caller).unwrap_or(false) {
                Ok(())
            } else {
                Err("Unauthorized to mint".into())
            }
        }

        fn ensure_can_burn(&self) -> Result<(), String> {
            let caller = self.env().caller();
            if caller == self.admin || 
               self.governance_address == Some(caller) ||
               self.authorized_burners.get(caller).unwrap_or(false) {
                Ok(())
            } else {
                Err("Unauthorized to burn".into())
            }
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
        pub fn max_supply(&self) -> Balance {
            self.max_supply
        }

        #[ink(message)]
        pub fn is_mintable(&self) -> bool {
            self.mintable
        }

        #[ink(message)]
        pub fn is_account_frozen(&self, account: AccountId) -> bool {
            self.frozen_accounts.get(account).unwrap_or(false)
        }

        #[ink(message)]
        pub fn is_authorized_minter(&self, account: AccountId) -> bool {
            self.authorized_minters.get(account).unwrap_or(false)
        }

        #[ink(message)]
        pub fn get_total_rewards_distributed(&self) -> Balance {
            self.total_rewards_distributed
        }
    }

    impl TokenInterface for PowergridToken {
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of(owner)
        }

        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance) -> bool {
            self.transfer(to, value).is_ok()
        }

        #[ink(message)]
        fn mint(&mut self, to: AccountId, value: Balance) -> bool {
            self.mint(to, value, "Cross-contract mint".into()).is_ok()
        }

        #[ink(message)]
        fn burn(&mut self, from: AccountId, value: Balance) -> bool {
            self.burn(from, value, "Cross-contract burn".into()).is_ok()
        }

        #[ink(message)]
        fn total_supply(&self) -> Balance {
            self.total_supply()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ink::env::test::{default_accounts, set_caller};

    #[ink::test]
    fn test_token_creation() {
        let accounts = default_accounts();
        let token = powergrid_token::PowergridToken::new(
            1_000_000_000,
            "PowerGrid".into(),
            "PWGD".into(),
            18,
            10_000_000_000,
        );

        assert_eq!(token.name(), "PowerGrid");
        assert_eq!(token.symbol(), "PWGD");
        assert_eq!(token.decimals(), 18);
        assert_eq!(token.total_supply(), 1_000_000_000);
        assert_eq!(token.max_supply(), 10_000_000_000);
        assert_eq!(token.balance_of(accounts.alice), 1_000_000_000);
    }

    #[ink::test]
    fn test_token_transfer() {
        let accounts = default_accounts();
        let mut token = powergrid_token::PowergridToken::new(
            1_000_000_000,
            "PowerGrid".into(),
            "PWGD".into(),
            18,
            10_000_000_000,
        );

        set_caller(accounts.alice);
        assert!(token.transfer(accounts.bob, 1000).is_ok());

        assert_eq!(token.balance_of(accounts.alice), 999_999_000);
        assert_eq!(token.balance_of(accounts.bob), 1000);
    }

    #[ink::test]
    fn test_token_staking() {
        let accounts = default_accounts();
        let mut token = powergrid_token::PowergridToken::new(
            1_000_000_000,
            "PowerGrid".into(),
            "PWGD".into(),
            18,
            10_000_000_000,
        );

        set_caller(accounts.alice);
        assert!(token.stake(50000).is_ok());

        assert_eq!(token.balance_of(accounts.alice), 999_950_000);
        assert_eq!(token.staked_balance_of(accounts.alice), 50000);
        assert_eq!(token.total_staked_supply(), 50000);
    }
}