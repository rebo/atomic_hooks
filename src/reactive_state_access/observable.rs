use crate::reactive_state_functions::STORE;
use crate::store::StorageKey;
use crate::{clone_reactive_state_with_id, read_reactive_state_with_id, ReactiveContext};
use std::cell::RefCell;

pub trait Observable<T>
where
    T: 'static,
{
    fn id(&self) -> StorageKey;
    fn observe(&self) -> T
    where
        T: Clone + 'static,
    {
        let context = illicit::get::<RefCell<ReactiveContext>>().expect(
            "No #[reaction] context found, are you sure you are in one? I.e. does the current \
             function have a #[reaction] tag?",
        );
        context
            .borrow_mut()
            .reactive_state_accessors
            .push(self.id());

        STORE.with(|store_refcell| {
            store_refcell
                .borrow_mut()
                .add_dependency(&self.id(), &context.borrow().key);
        });

        clone_reactive_state_with_id::<T>(self.id()).unwrap()
    }
    fn observe_update(&self) -> (Option<T>, T)
    where
        T: Clone + 'static;
    fn observe_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        if let Ok(context) = illicit::get::<RefCell<ReactiveContext>>() {
            context
                .borrow_mut()
                .reactive_state_accessors
                .push(self.id());

            STORE.with(|store_refcell| {
                store_refcell
                    .borrow_mut()
                    .add_dependency(&self.id(), &context.borrow().key);
            });
        }
        read_reactive_state_with_id(self.id(), func)
    }
}
