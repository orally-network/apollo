use std::borrow::{Borrow, BorrowMut};

use apollo_utils::{
    address,
    errors::BalancesError,
    get_metadata, log,
    memory::Cbor,
    multicall::{BASE_GAS, GAS_PER_TRANSFER},
};
use candid::{CandidType, Nat};

use anyhow::Result;
use ic_stable_structures::StableBTreeMap;
use serde::{Deserialize, Serialize};

use crate::memory::VMemory;

use super::STATE;

const ETH_TRANSFER_GAS_LIMIT: u64 = BASE_GAS + GAS_PER_TRANSFER; // TODO: recheck this value, calculate from actual tx

#[derive(Clone, Debug, Default, CandidType, Serialize, Deserialize)]
pub struct UserBalance {
    pub amount: Nat,
    pub last_nonce: Nat,
}

/// chain id => user's public key => PUB (Pythia User Balance)
pub struct Balances(StableBTreeMap<String, Cbor<UserBalance>, VMemory>);

// pub struct Balances(StableCell<Cbor<HashMap<String, UserBalance>>, VMemory>);

impl Default for Balances {
    fn default() -> Self {
        Self(StableBTreeMap::init(crate::memory::get_balances_memory()))
    }
}

impl Balances {
    pub fn get_value_for_withdraw(address: &str, gas_price: &Nat) -> Result<Nat> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            // let balances = state.balances.get().0;
            // let balance = balances.0.get_mut(chain_id).;
            todo!()

            // let gas = Nat::from(ETH_TRANSFER_GAS_LIMIT) * gas_price.clone();
            // if balance.amount < gas {
            //     return Err(anyhow!("not enough funds to pay for gas"));
            // }
            // let value = balance.amount.clone() - gas;
            // balance.amount = Nat::from(0);
            // Ok(value)
        })
    }

    pub fn save_nonce(address: &str, nonce: &Nat) -> Result<(), BalancesError> {
        let address = address::normalize(address)?;
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let inner = state.balances.0.borrow_mut();

            let mut balance = inner.get(&address).unwrap_or_default();

            if &balance.last_nonce >= nonce {
                return Err(BalancesError::NonceIsTooLow);
            }
            balance.last_nonce = nonce.clone();

            inner.insert(address, balance);
            Ok(())
        })
    }

    pub fn add_amount(address: &str, amount: &Nat) -> Result<(), BalancesError> {
        let address = address::normalize(address)?;
        let chain_id = get_metadata!(chain_id);

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let inner = state.balances.0.borrow_mut();

            let mut balance = inner.get(&address).unwrap_or_default();

            balance.amount += amount.clone();

            inner.insert(address.clone(), balance);

            log!(
                "[BALANCES] Balance amount added: chain_id = {}, address = {}, amount = {}",
                chain_id,
                address,
                amount
            );

            Ok(())
        })
    }

    pub fn reduce_amount(address: &str, amount: &Nat) -> Result<(), BalancesError> {
        let address = address::normalize(address)?;
        let chain_id = get_metadata!(chain_id);

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let inner = state.balances.0.borrow_mut();

            let mut balance = inner.get(&address).unwrap_or_default();

            if &balance.amount < amount {
                return Err(BalancesError::NotEnoughFunds);
            }

            balance.amount -= amount.clone();

            inner.insert(address.clone(), balance);

            log!(
                "[BALANCES] Balance amount reduced: chain_id = {}, address = {}, amount = {}",
                chain_id,
                address,
                amount
            );

            Ok(())
        })
    }

    pub fn get(address: &str) -> Result<UserBalance, BalancesError> {
        let address = address::normalize(address)?;
        STATE.with(|state| {
            let state = state.borrow();
            let inner = state.balances.0.borrow();

            Ok(inner
                .get(&address)
                .map(|user_balance| (*user_balance).clone())
                .unwrap_or_default())
        })
    }

    pub fn is_sufficient(address: &str, amount: &Nat) -> Result<bool> {
        let address = address::normalize(address)?;

        let balance = STATE.with(|state| {
            let state = state.borrow();
            let inner = state.balances.0.borrow();

            inner.get(&address).unwrap_or_default()
        });

        Ok(&balance.amount >= amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce() -> anyhow::Result<()> {
        Balances::save_nonce("0x89A4e2Cf7F72b6e462bbA27FEa4d40c3da1d46cd", &Nat::from(1))?;

        let user_balances = Balances::get("0x89A4e2Cf7F72b6e462bbA27FEa4d40c3da1d46cd")?;

        assert_eq!(user_balances.last_nonce, Nat::from(1));

        let result =
            Balances::save_nonce("0x89A4e2Cf7F72b6e462bbA27FEa4d40c3da1d46cd", &Nat::from(1));

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), BalancesError::NonceIsTooLow);

        Ok(())
    }

    #[test]
    fn test_changing_amount() -> anyhow::Result<()> {
        Balances::add_amount(
            "0x89A4e2Cf7F72b6e462bbA27FEa4d40c3da1d46cd",
            &Nat::from(1234567890),
        )?;

        let user_balances = Balances::get("0x89A4e2Cf7F72b6e462bbA27FEa4d40c3da1d46cd")?;

        assert_eq!(user_balances.amount, Nat::from(1234567890));

        Balances::reduce_amount(
            "0x89A4e2Cf7F72b6e462bbA27FEa4d40c3da1d46cd",
            &Nat::from(1234567890),
        )?;

        let user_balances = Balances::get("0x89A4e2Cf7F72b6e462bbA27FEa4d40c3da1d46cd")?;

        assert_eq!(user_balances.amount, Nat::from(0));

        let result =
            Balances::reduce_amount("0x89A4e2Cf7F72b6e462bbA27FEa4d40c3da1d46cd", &Nat::from(1));

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), BalancesError::NotEnoughFunds);

        Ok(())
    }
}
