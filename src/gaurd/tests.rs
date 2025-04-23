#[cfg(test)]
mod tests {
    use crate::gaurd::{mutate_guarded_principals, PrincipalGuard, PrincipalGuardError};

    use candid::Principal;

    // Helper to create test principals
    fn test_principal(id: u64) -> Principal {
        Principal::from_slice(&id.to_le_bytes())
    }

    #[test]
    fn test_guard_acquire_and_release() {
        let principal = test_principal(1);

        // Acquire guard
        let guard = PrincipalGuard::new(principal).expect("Should acquire guard");
        assert_eq!(
            mutate_guarded_principals(|p| p.contains(&principal)),
            true,
            "Principal should be guarded"
        );

        // Attempt to acquire another guard for the same principal
        let result = PrincipalGuard::new(principal);
        assert_eq!(
            result,
            Err(PrincipalGuardError::AlreadyProcessing),
            "Should prevent duplicate guard"
        );

        // Drop the guard explicitly
        drop(guard);
        assert_eq!(
            mutate_guarded_principals(|p| p.contains(&principal)),
            false,
            "Principal should be released"
        );

        // Acquire a new guard after release
        let new_guard = PrincipalGuard::new(principal).expect("Should acquire guard after release");
        assert_eq!(
            mutate_guarded_principals(|p| p.contains(&principal)),
            true,
            "Principal should be guarded again"
        );
    }

    #[test]
    fn test_multiple_principals() {
        let principal1 = test_principal(1);
        let principal2 = test_principal(2);

        // Acquire guard for principal1
        let guard1 = PrincipalGuard::new(principal1).expect("Should acquire guard for principal1");
        assert_eq!(
            mutate_guarded_principals(|p| p.contains(&principal1)),
            true,
            "Principal1 should be guarded"
        );

        // Acquire guard for principal2
        let guard2 = PrincipalGuard::new(principal2).expect("Should acquire guard for principal2");
        assert_eq!(
            mutate_guarded_principals(|p| p.contains(&principal2)),
            true,
            "Principal2 should be guarded"
        );

        // principal1 cannot acquire another guard
        assert_eq!(
            PrincipalGuard::new(principal1),
            Err(PrincipalGuardError::AlreadyProcessing),
            "Principal1 should be blocked"
        );

        // Drop guards
        drop(guard1);
        drop(guard2);

        // Both principals should be released
        assert_eq!(
            mutate_guarded_principals(|p| p.contains(&principal1)),
            false,
            "Principal1 should be released"
        );
        assert_eq!(
            mutate_guarded_principals(|p| p.contains(&principal2)),
            false,
            "Principal2 should be released"
        );

        // Both can acquire new guards
        let _guard1 = PrincipalGuard::new(principal1).expect("Principal1 should acquire guard");
        let _guard2 = PrincipalGuard::new(principal2).expect("Principal2 should acquire guard");
    }

    #[test]
    fn test_guard_scope_release() {
        let principal = test_principal(1);

        // Guard acquired and released within a scope
        {
            let _guard = PrincipalGuard::new(principal).expect("Should acquire guard");
            assert_eq!(
                mutate_guarded_principals(|p| p.contains(&principal)),
                true,
                "Principal should be guarded"
            );
        }

        // Guard should be released after scope exit
        assert_eq!(
            mutate_guarded_principals(|p| p.contains(&principal)),
            false,
            "Principal should be released"
        );

        // Can acquire a new guard
        let _guard = PrincipalGuard::new(principal).expect("Should acquire guard after scope");
    }

    #[test]
    fn test_nested_operation_prevented() {
        let principal = test_principal(1);

        // Simulate an operation
        let guard = PrincipalGuard::new(principal).expect("Should acquire guard");

        // Simulate a nested operation by the same principal
        let result = PrincipalGuard::new(principal);
        assert_eq!(
            result,
            Err(PrincipalGuardError::AlreadyProcessing),
            "Nested operation should be blocked"
        );

        // Drop guard to complete operation
        drop(guard);

        // New operation should succeed
        let _guard = PrincipalGuard::new(principal).expect("Should acquire guard after operation");
        assert_eq!(
            mutate_guarded_principals(|p| p.contains(&principal)),
            true,
            "Principal should be guarded again"
        );
    }
}
