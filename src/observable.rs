use crate::reactive_state_access::state_access::{CloneState, StateAccess};

use std::cell::RefCell;

use crate::{
    reactive_state_functions::{clone_reactive_state_with_id, read_reactive_state_with_id, STORE},
    store::ReactiveContext,
};

pub trait Observable<T>
where
    T: 'static,
{
    // fn id(&self) -> StorageKey;
    fn observe(&self) -> T
    where
        T: Clone + 'static;
    fn observe_update(&self) -> (Option<T>, T)
    where
        T: Clone + 'static;
    fn observe_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R;
}
