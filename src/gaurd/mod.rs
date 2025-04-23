#[cfg(test)]
mod tests;

use candid::Principal;
use std::{cell::RefCell, collections::BTreeSet};

thread_local! {
    pub static GUARDED_PRINCIPALS:RefCell<Option<BTreeSet<Principal>>>=RefCell::new(Some(BTreeSet::default()));
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_guarded_principals<F, R>(f: F) -> R
where
    F: FnOnce(&mut BTreeSet<Principal>) -> R,
{
    GUARDED_PRINCIPALS.with(|s| {
        f(s.borrow_mut()
            .as_mut()
            .expect("Bug: principal gurd should be initialized"))
    })
}

#[derive(Debug, PartialEq, Eq)]
pub enum PrincipalGuardError {
    AlreadyProcessing,
}

/// Guards a block from executing twice when called by the same user and from being
#[derive(Debug, PartialEq, Eq)]
pub struct PrincipalGuard {
    pub principal: Principal,
}

impl PrincipalGuard {
    pub fn new(principal: Principal) -> Result<Self, PrincipalGuardError> {
        mutate_guarded_principals(|active_principals| {
            if !active_principals.insert(principal) {
                return Err(PrincipalGuardError::AlreadyProcessing);
            }
            Ok(Self { principal })
        })
    }
}

impl Drop for PrincipalGuard {
    fn drop(&mut self) {
        mutate_guarded_principals(|active_principals| {
            active_principals.remove(&self.principal);
        });
    }
}
