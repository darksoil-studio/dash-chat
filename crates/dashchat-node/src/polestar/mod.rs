//! Types to use with the polestar model checker

mod action;

pub use action::*;

pub fn emit(_event: Action) {
    // tracing::info!(target: "polestar", "event: {:?}", event);
}
