#![feature(track_caller)]
pub mod prelude;
mod reactive_state_access;
mod reactive_state_functions;
mod reactive_store;
mod local_update_el;
// mod helpers;
// mod state_access;
// mod state_functions;
// mod store;
// mod hybrid_store;
// mod unmount;

pub use prelude::*;




#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
