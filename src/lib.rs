pub mod prelude;
mod atom_state_access;
mod atom_state_functions;
mod atom_store;
pub use prelude::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
