#[cfg(test)]
mod tests;

use candid::Principal;
use std::{cell::RefCell, collections::HashSet};

/// A guard to prevent double-spending or concurrent non-swap operations for a principal.
/// Swap operations can run concurrently with unique swap numbers, but other operations
/// are mutually exclusive with all operations for the same principal.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PrincipalGuard {
    lock: Guard,
}

/// The internal state of a guard, stored in the global set.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Guard {
    principal: Principal,
    is_swap_guard: bool,
    swap_number: Option<u32>,
}

thread_local! {
    static GUARDED_PRINCIPALS: RefCell<HashSet<Guard>> = RefCell::new(HashSet::default());
}

/// Errors that can occur when creating a principal guard.
#[derive(Debug, PartialEq, Eq)]
pub enum PrincipalGuardError {
    AlreadyProcessing { principal: Principal },
}

/// Mutates the guarded principals set using the provided closure.
pub fn mutate_guarded_principals<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashSet<Guard>) -> R,
{
    GUARDED_PRINCIPALS.with(|s| f(&mut s.borrow_mut()))
}

impl PrincipalGuard {
    /// Creates a new swap guard for a principal.
    /// Fails if a non-swap guard exists for the principal.
    pub fn new_swap_guard(principal: Principal) -> Result<Self, PrincipalGuardError> {
        mutate_guarded_principals(|guards| {
            // Check for non-swap guards
            if guards
                .iter()
                .any(|g| g.principal == principal && !g.is_swap_guard)
            {
                return Err(PrincipalGuardError::AlreadyProcessing { principal });
            }

            // Calculate next swap number
            let next_swap_number = guards
                .iter()
                .filter(|g| g.principal == principal && g.is_swap_guard)
                .filter_map(|g| g.swap_number)
                .max()
                .map_or(0, |n| n.saturating_add(1));

            let guard = Guard {
                principal,
                is_swap_guard: true,
                swap_number: Some(next_swap_number),
            };

            guards.insert(guard.clone());
            Ok(PrincipalGuard { lock: guard })
        })
    }

    /// Creates a new general guard for a principal.
    /// Fails if any guard (swap or non-swap) exists for the principal.
    pub fn new_general_guard(principal: Principal) -> Result<Self, PrincipalGuardError> {
        mutate_guarded_principals(|guards| {
            if guards.iter().any(|g| g.principal == principal) {
                return Err(PrincipalGuardError::AlreadyProcessing { principal });
            }

            let guard = Guard {
                principal,
                is_swap_guard: false,
                swap_number: None,
            };

            guards.insert(guard.clone());
            Ok(PrincipalGuard { lock: guard })
        })
    }

    /// Returns the principal associated with the guard.
    pub fn principal(&self) -> Principal {
        self.lock.principal
    }

    /// Returns whether the guard is a swap guard.
    pub fn is_swap_guard(&self) -> bool {
        self.lock.is_swap_guard
    }

    /// Returns the swap number, if the guard is a swap guard.
    pub fn swap_number(&self) -> Option<u32> {
        self.lock.swap_number
    }
}

impl Drop for PrincipalGuard {
    fn drop(&mut self) {
        mutate_guarded_principals(|guards| {
            guards.remove(&self.lock);
        });
    }
}
