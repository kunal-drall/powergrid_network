#[cfg(test)]
mod integration_tests {
    use ink::env::test::default_accounts;
    use super::*;

    #[ink::test]
    fn full_flow_works() {
        let accounts = default_accounts::<ink::env::DefaultEnvironment>();
        let mut token = token::Token::new(1000000);
        let token_address = accounts.bob;
        let mut registry = resource_registry::ResourceRegistry::new(100);
        let mut grid = grid_service::GridService::new(token_address);
        let mut governance = governance::Governance::new(token_address);

        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(100);
        registry.register_device("SmartPlug".into(), 1000, "Delhi".into());

        grid.create_event("DemandResponse".into(), 60, 10);
        grid.participate(0);

        token.mint(accounts.alice, 100);

        governance.create_proposal("Update min_stake".into());
        governance.vote(0, true);

        assert_eq!(registry.get_device(accounts.alice).unwrap().capacity, 1000);
        assert!(grid.participation.get(&(0, accounts.alice)).is_some());
        assert_eq!(token.balance_of(accounts.alice), 100);
        assert_eq!(governance.proposals.get(&0).unwrap().yes_votes, 1);
    }
}