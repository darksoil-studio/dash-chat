mod action;

pub use action::*;

pub fn emit(event: Action) {
    // tracing::info!(target: "polestar", "event: {:?}", event);
}
