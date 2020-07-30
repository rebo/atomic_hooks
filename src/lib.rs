pub use atomic_hooks_macros::{atom, reaction};
// storage
pub mod store;

// hooks
mod state_access;
mod hooks_state_functions;

// reactive state

mod reactive_state_access;
pub mod reactive_state_functions;
mod marker;

// helpers
mod helpers;
mod observable;
// mod seed_integration;
pub mod undo;

// public exports
mod prelude;
pub use prelude::*;
pub mod unmount;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
