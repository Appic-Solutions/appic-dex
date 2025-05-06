#[cfg(test)]
mod tests {
    use crate::guard::{mutate_guarded_principals, PrincipalGuard, PrincipalGuardError};

    use candid::Principal;

    // Helper to create a Principal from a byte slice
    fn create_principal(id: u8) -> Principal {
        Principal::from_slice(&[id; 29])
    }

    // Clear the guarded principals set before each test
    fn clear_guards() {
        mutate_guarded_principals(|guards| guards.clear());
    }

    #[test]
    fn test_new_swap_guard_success() {
        clear_guards();
        let principal = create_principal(1);
        let guard = PrincipalGuard::new_swap_guard(principal).unwrap();
        assert_eq!(guard.principal(), principal);
        assert!(guard.is_swap_guard());
        assert_eq!(guard.swap_number(), Some(0));

        // Verify another swap guard can be created
        let guard2 = PrincipalGuard::new_swap_guard(principal).unwrap();
        assert_eq!(guard2.swap_number(), Some(1));
    }

    #[test]
    fn test_new_general_guard_success() {
        clear_guards();
        let principal = create_principal(1);
        let guard = PrincipalGuard::new_general_guard(principal).unwrap();
        assert_eq!(guard.principal(), principal);
        assert!(!guard.is_swap_guard());
        assert_eq!(guard.swap_number(), None);

        // Verify general guard blocks another general guard
        assert_eq!(
            PrincipalGuard::new_general_guard(principal),
            Err(PrincipalGuardError::AlreadyProcessing { principal })
        );
    }

    #[test]
    fn test_multiple_swap_guards() {
        clear_guards();
        let principal = create_principal(1);
        let guard1 = PrincipalGuard::new_swap_guard(principal).unwrap();
        let guard2 = PrincipalGuard::new_swap_guard(principal).unwrap();

        assert_eq!(guard1.swap_number(), Some(0));
        assert_eq!(guard2.swap_number(), Some(1));
    }

    #[test]
    fn test_general_guard_blocks_all() {
        clear_guards();
        let principal = create_principal(1);
        let _guard = PrincipalGuard::new_general_guard(principal).unwrap();

        assert_eq!(
            PrincipalGuard::new_swap_guard(principal),
            Err(PrincipalGuardError::AlreadyProcessing { principal })
        );
        assert_eq!(
            PrincipalGuard::new_general_guard(principal),
            Err(PrincipalGuardError::AlreadyProcessing { principal })
        );
    }

    #[test]
    fn test_swap_guard_blocks_general() {
        clear_guards();
        let principal = create_principal(1);
        let _guard = PrincipalGuard::new_swap_guard(principal).unwrap();

        assert_eq!(
            PrincipalGuard::new_general_guard(principal),
            Err(PrincipalGuardError::AlreadyProcessing { principal })
        );
        let guard2 = PrincipalGuard::new_swap_guard(principal).unwrap();
        assert_eq!(guard2.swap_number(), Some(1));
    }

    #[test]
    fn test_drop_cleans_up() {
        clear_guards();
        let principal = create_principal(1);
        {
            let _guard = PrincipalGuard::new_swap_guard(principal).unwrap();
            assert_eq!(
                PrincipalGuard::new_general_guard(principal),
                Err(PrincipalGuardError::AlreadyProcessing { principal })
            );
        }
        let _guard = PrincipalGuard::new_general_guard(principal).unwrap();
    }

    #[test]
    fn test_different_principals() {
        clear_guards();
        let principal1 = create_principal(1);
        let principal2 = create_principal(2);

        let _guard1 = PrincipalGuard::new_general_guard(principal1).unwrap();
        let _guard2 = PrincipalGuard::new_swap_guard(principal2).unwrap();

        assert_eq!(
            PrincipalGuard::new_general_guard(principal1),
            Err(PrincipalGuardError::AlreadyProcessing {
                principal: principal1
            })
        );
        assert_eq!(
            PrincipalGuard::new_general_guard(principal2),
            Err(PrincipalGuardError::AlreadyProcessing {
                principal: principal2
            })
        );
    }

    #[test]
    fn test_swap_number_increment() {
        clear_guards();
        let principal = create_principal(1);
        let guard = PrincipalGuard::new_swap_guard(principal).unwrap();
        assert_eq!(guard.swap_number(), Some(0));

        let guard1 = PrincipalGuard::new_swap_guard(principal).unwrap();
        assert_eq!(guard1.swap_number(), Some(1));

        let guard2 = PrincipalGuard::new_swap_guard(principal).unwrap();
        assert_eq!(guard2.swap_number(), Some(2));

        let guard3 = PrincipalGuard::new_swap_guard(principal).unwrap();
        assert_eq!(guard3.swap_number(), Some(3));

        let guard4 = PrincipalGuard::new_swap_guard(principal).unwrap();
        assert_eq!(guard4.swap_number(), Some(4));
    }

    #[test]
    fn test_borrow_safety() {
        clear_guards();
        let principal = create_principal(1);

        let _guard1 = PrincipalGuard::new_swap_guard(principal).unwrap();
        let _guard2 = PrincipalGuard::new_swap_guard(principal).unwrap();
        let _guard3 = PrincipalGuard::new_swap_guard(principal).unwrap();

        assert_eq!(
            PrincipalGuard::new_general_guard(principal),
            Err(PrincipalGuardError::AlreadyProcessing { principal })
        );
    }
}
