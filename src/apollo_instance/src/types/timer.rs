use anyhow::Result;
use candid::CandidType;
use ic_cdk_timers::{clear_timer, TimerId};
use serde::{Deserialize, Serialize};

use crate::{log, STATE};

#[derive(Clone, Debug, Default, Serialize, Deserialize, CandidType)]
pub struct Timer {
    pub id: String,
    pub is_active: bool,
}

impl Timer {
    pub fn update(id: TimerId) -> Result<()> {
        let id = serde_json::to_string(&id)?;
        STATE.with(|state| {
            let mut state = state.borrow_mut();

            let old_timer = state.timer.clone();

            let new_timer = Timer {
                id,
                is_active: old_timer.is_active,
            };

            log!("[TIMER] Timer updated, is_active = {}", new_timer.is_active);

            state.timer = new_timer;

            Ok(())
        })
    }

    pub fn activate() {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let old_timer = state.timer.clone();

            let new_timer = Timer {
                id: old_timer.id,
                is_active: true,
            };

            log!("[TIMER] Timer activated");

            state.timer = new_timer;
        })
    }

    pub fn deactivate() -> Result<()> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();

            let old_timer = state.timer.clone();

            let new_timer = Timer {
                id: old_timer.id,
                is_active: false,
            };

            let id = serde_json::from_str::<TimerId>(&state.timer.id)?;

            clear_timer(id);

            log!("[TIMER] Timer deactivated");

            state.timer = new_timer;

            Ok(())
        })
    }

    pub fn is_active() -> bool {
        STATE.with(|state| {
            let state = state.borrow();

            state.timer.is_active
        })
    }
}
