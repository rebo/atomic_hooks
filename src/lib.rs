pub mod prelude;
mod atomic_state_access;
mod atomic_state_functions;
mod atomic_store;
pub use prelude::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
