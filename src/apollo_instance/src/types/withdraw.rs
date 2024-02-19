use std::borrow::Cow;

use anyhow::Result;

use apollo_utils::{errors::WithdrawRequestsError, memory::Cbor};
use candid::{CandidType, Nat};
use ic_stable_structures::{storable::Bound, StableVec, Storable};
use serde::{Deserialize, Serialize};

use crate::{log, memory::VMemory, STATE};

#[derive(Clone, Debug, Default, CandidType, Serialize, Deserialize)]
pub struct WithdrawRequest {
    pub amount: Nat, // 8 + 8 + 8 + 8  - approximated, these numbers has been obtainet from raw data (WithdrawRequest has always been returning 114 bytes)
    pub receiver: String, // 42 - length of the address
    pub from: String, // 42 - length of the address
}

// implementing Storable for WithdrawRequest
// because Cbor wrapper is Bound::Unbounded
// and we need WithdrawRequest to be Bound::Bounded for StableVec
impl Storable for WithdrawRequest {
    const BOUND: Bound = Bound::Bounded {
        max_size: 8 + 8 + 8 + 8 + 42 + 42,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![];
        ciborium::ser::into_writer(&self, &mut buf).unwrap();

        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref()).unwrap()
    }
}

pub struct WithdrawRequests(StableVec<WithdrawRequest, VMemory>);

impl Default for WithdrawRequests {
    fn default() -> Self {
        // TODO: change to StableVec::init
        Self(StableVec::new(crate::memory::get_withdraw_requests_memory()).unwrap())
    }
}

impl WithdrawRequests {
    pub fn add(from: String, receiver: String, amount: &Nat) -> Result<(), WithdrawRequestsError> {
        STATE.with(|state| {
            state
                .borrow_mut()
                .withdraw_requests
                .0
                .push(&Cbor(WithdrawRequest {
                    amount: amount.clone(),
                    receiver: receiver.clone(),
                    from: from.clone(),
                }))
                .map_err(|err| {
                    WithdrawRequestsError::UnableToAddWithdrawRequest(err.to_string())
                })?;

            log!(
                "[WITHDRAWER] Withdraw request added: amount = {}, receiver = {}, from = {}",
                amount,
                receiver,
                from
            );

            Ok(())
        })
    }

    pub fn get_all() -> Vec<WithdrawRequest> {
        STATE.with(|state| {
            state
                .borrow()
                .withdraw_requests
                .0
                .iter()
                .map(|req| req.clone())
                .collect()
        })
    }

    pub fn clean() -> Result<(), WithdrawRequestsError> {
        STATE.with(|state| {
            state.borrow_mut().withdraw_requests.0 =
                StableVec::new(crate::memory::get_withdraw_requests_memory()).map_err(|err| {
                    WithdrawRequestsError::UnableToCleanWithdrawRequests(err.to_string())
                })?;

            log!("[WITHDRAWER] Withdraw request removed");
            Ok(())
        })
    }
}
