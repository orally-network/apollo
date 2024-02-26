use std::borrow::{Borrow, BorrowMut};

use apollo_utils::{address, errors::UtilsError};
use ic_stable_structures::StableBTreeMap;

use crate::memory::VMemory;

use super::STATE;

// Allowances is a map that contains which contracts are allowed to use which user's balance
// contract public key => user public key
pub struct Allowances(StableBTreeMap<String, String, VMemory>);

impl Default for Allowances {
    fn default() -> Self {
        Self(StableBTreeMap::init(crate::memory::get_allowances_memory()))
    }
}

impl Allowances {
    pub fn grant(contract: String, user: String) -> Result<(), UtilsError> {
        let contract = address::normalize(&contract)?;
        let user = address::normalize(&user)?;

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let inner = state.allowances.0.borrow_mut();

            if inner.get(&contract).is_none() {
                inner.insert(contract, user);
            }
        });

        Ok(())
    }

    pub fn restrict(contract: String, user: String) -> Result<(), UtilsError> {
        let contract = address::normalize(&contract)?;
        let user = address::normalize(&user)?;

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let inner = state.allowances.0.borrow_mut();

            if inner.get(&contract) == Some(user) {
                inner.remove(&contract);
            }
        });

        Ok(())
    }

    /// Returns the user's pubkey if the contract is allowed to use his balance, otherwise returns the contract's pubkey
    pub fn get_allowed_user(contract: String) -> Result<String, UtilsError> {
        let contract = address::normalize(&contract)?;

        STATE.with(|state| {
            let state = state.borrow();
            let inner = state.allowances.0.borrow();

            Ok(inner.get(&contract).unwrap_or(contract))
        })
    }
}
