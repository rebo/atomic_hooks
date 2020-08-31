use crate::reactive_state_access::state_access::CloneState;
use crate::reactive_state_access::CloneReactiveState;
use crate::reactive_state_functions::STORE;
use crate::{
    clone_reactive_state_with_id, reactive_state_exists_for_id,
    reactive_state_functions::{
        execute_reaction_nodes, remove_reactive_state_with_id_with_undo,
        set_atom_state_with_id_with_undo, update_atom_state_with_id_with_undo,
    },
    read_reactive_state_with_id, set_inert_atom_state_with_id_with_undo,
    store::StorageKey,
    Observable, ReactiveContext, RxFunc,
};
use std::cell::RefCell;
use std::marker::PhantomData;

///
/// An AtomUndo is similar to a regular atom except that it is reversible and
/// is stored in a global state.
/// ```
/// use atomic_hooks_macros::*;
/// use store::RxFunc;
/// use atomic_hooks::{global_undo_queue, AtomUndo, GlobalUndo,
/// CloneReactiveState};
/// use atomic_hooks::atom_undo::AtomUndo;
///
/// #[atom(undo)]
/// fn a() -> AtomUndo<i32> {
///     0
/// }
///
/// #[atom(undo)]
/// fn b() -> AtomUndo<i32> {
///    0
/// }
///
/// fn test_undo() {
///   a().set(3);
///
///   a().set(5);
///
///   b().set(10);
///
///   a().set(4);
///
///     assert_eq!(a().get(), 4, "We should get 4 as value for a");
///
///     global_undo_queue().travel_backwards();
///     assert_eq!(b().get(), 10, "We should get 10 as value for b");
///
///     global_undo_queue().travel_backwards();
///     assert_eq!(a().get(), 5, "We should get 5 as value for a");
///
///     global_undo_queue().travel_backwards();
///     assert_eq!(a().get(), 3, "We should get 3 as value for a");
///
///     global_undo_queue().travel_backwards();
///     assert_eq!(a().get(), 0, "We should get 0 as value for a");
/// }
///  ```
pub struct AtomUndo<T>
where
    T: Clone,
{
    pub id: StorageKey,
    pub _phantom_data_stored_type: PhantomData<T>,
}

impl<T> std::fmt::Debug for AtomUndo<T>
where
    T: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:#?})", self.id)
    }
}

impl<T> Clone for AtomUndo<T>
where
    T: Clone,
{
    fn clone(&self) -> AtomUndo<T> {
        AtomUndo::<T> {
            id: self.id,

            _phantom_data_stored_type: PhantomData::<T>,
        }
    }
}

impl<T> Observable<T> for AtomUndo<T>
where
    T: 'static + Clone,
{
    fn observe(&self) -> T
    where
        T: 'static + Clone,
    {
        let context = illicit::get::<RefCell<ReactiveContext>>().expect(
            "No #[reaction] context found, are you sure you are in one? I.e. does the current \
             function have a #[reaction] tag?",
        );
        context.borrow_mut().reactive_state_accessors.push(self.id);

        STORE.with(|store_refcell| {
            store_refcell
                .borrow_mut()
                .add_dependency(&self.id, &context.borrow().key);
        });

        clone_reactive_state_with_id::<T>(self.id).unwrap()
    }

    #[topo::nested]
    fn observe_update(&self) -> (Option<T>, T)
    where
        T: 'static + Clone,
    {
        let previous_value_access = crate::hooks_state_functions::use_state(|| None);
        let opt_previous_value = previous_value_access.get();
        let new_value = self.get();
        previous_value_access.set(Some(new_value.clone()));
        (opt_previous_value, new_value)
    }

    // <T: 'static, F: FnOnce(&T) -> R, R>(id: StorageKey, func: F) -> R {
    fn observe_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        if let Ok(context) = illicit::get::<RefCell<ReactiveContext>>() {
            context.borrow_mut().reactive_state_accessors.push(self.id);

            STORE.with(|store_refcell| {
                store_refcell
                    .borrow_mut()
                    .add_dependency(&self.id, &context.borrow().key);
            });
        }
        read_reactive_state_with_id(self.id, func)
    }
    fn id(&self) -> StorageKey {
        self.id
    }
}

impl<T> Copy for AtomUndo<T> where T: Clone {}

impl<T> AtomUndo<T>
where
    T: 'static + Clone,
{
    pub fn new(id: StorageKey) -> AtomUndo<T> {
        AtomUndo {
            id,
            _phantom_data_stored_type: PhantomData,
        }
    }

    /// Stores a value of type T in a backing Store **without** reaction for
    /// observers.
    ///
    /// ```
    /// use atomic_hooks::atom_undo::AtomUndo;
    /// use atomic_hooks::Observable;
    /// use atomic_hooks::reaction::Reaction;
    /// #[atom(undo)]
    /// fn a() -> AtomUndo<i32> {
    ///     0
    /// }
    /// #[atom(undo)]
    /// fn b() -> AtomUndo<i32> {
    ///     0
    /// }
    ///
    /// #[reaction]
    /// fn reaction_a_b_subtraction() -> Reaction<i32> {
    ///     let a = a().observe();
    ///     let b = b().observe();
    ///     a - b
    /// }
    ///
    /// a().inert_set(1);
    /// let diff = reaction_a_b_subtraction();
    /// assert_eq!(a().get(), 1);
    /// assert_eq!(
    ///     diff.get(),
    ///     0,
    ///     "We should still get 0 since we use inert setting"
    /// );
    /// ```
    ///  ## Todo doc
    /// - need to add description when the use of this method is relevant.
    pub fn inert_set(self, value: T)
    where
        T: 'static,
    {
        set_inert_atom_state_with_id_with_undo(value, self.id);
    }
    /// ```
    /// use atomic_hooks::atom_undo::AtomUndo;
    /// #[atom(undo)]
    /// fn a() -> AtomUndo<i32> {
    ///     0
    /// }
    ///
    /// a().set(1);
    ///
    /// assert_eq!(a().get(), 1);
    /// ```
    /// - add example maybe
    /// - When to use it
    pub fn set(self, value: T)
    where
        T: 'static,
    {
        set_atom_state_with_id_with_undo(value, self.id);
    }
    /// This is use for example when we want to update a component rendering
    /// depending of a state. We update the atom so the component will
    /// rerender with the new state. If many components subscribed to the
    /// atom, then all of them will get the update.
    /// ```
    /// use atomic_hooks::atom_undo::AtomUndo;
    /// #[atom(undo)]
    /// fn a() -> AtomUndo<i32> {
    ///     0
    /// }
    /// a().update(|state| *state = 45);
    /// assert_eq!(a().get(), 45, "We should get 45 as value for a");
    /// ```
    pub fn update<F: FnOnce(&mut T) -> ()>(&self, func: F)
    where
        T: 'static,
    {
        update_atom_state_with_id_with_undo(self.id, func);
    }

    /// ```
    ///
    /// use atomic_hooks::atom_undo::AtomUndo;
    /// #[atom(undo)]
    /// fn a() -> AtomUndo<i32> {
    ///     0
    /// }
    ///
    /// a().remove();
    ///
    /// assert_eq!(a().state_exists(), false, "The a state should not exist");
    /// ```
    pub fn remove(self) -> Option<T> {
        remove_reactive_state_with_id_with_undo(self.id)
    }
    /// ## Question :
    /// Why do we have remove and delete ?
    ///
    /// ```
    ///
    /// use atomic_hooks::atom_undo::AtomUndo;
    /// #[atom(undo)]
    /// fn a() -> AtomUndo<i32> {
    ///     0
    /// }
    ///
    /// a().delete();
    ///
    /// assert_eq!(a().state_exists(), false, "The a state should not exist");
    /// ```
    pub fn delete(self) {
        self.remove();
    }
    /// Reset to the initial value.
    /// ```
    ///
    ///
    /// use atomic_hooks::atom_undo::AtomUndo;
    /// #[atom(undo)]
    /// fn a() -> AtomUndo<i32> {
    ///     0
    /// }
    ///
    /// a().set(10);
    /// a().reset_to_default();
    ///
    /// assert_eq!(a().get(), 0, "The a state be reset to initial value");
    /// ```
    pub fn reset_to_default(&self) {
        (clone_reactive_state_with_id::<RxFunc>(self.id)
            .unwrap()
            .func)();
        execute_reaction_nodes(&self.id);
    }
    /// ```
    ///
    ///
    /// use atomic_hooks::atom_undo::AtomUndo;
    /// #[atom(undo)]
    /// fn a() -> AtomUndo<i32> {
    ///     0
    /// }
    ///
    /// a().set(10);
    /// a().delete();
    ///
    /// assert_eq!(a().state_exists(), false, "The a state should not exist");
    /// ```
    pub fn state_exists(self) -> bool {
        reactive_state_exists_for_id::<T>(self.id)
    }

    /// Allow you to get the state through a reference with a closure.
    /// ```
    ///
    /// use atomic_hooks::atom_undo::AtomUndo;
    /// #[atom(undo)]
    /// fn a() -> AtomUndo<i32> {
    ///     0
    /// }
    /// a().set(3);
    ///
    /// a().get_with(|v| assert_eq!(v, &3, "We should get 3"));
    /// ```
    ///  ## Todo doc
    /// - When to use it ?
    pub fn get_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        read_reactive_state_with_id(self.id, func)
    }

    // #[topo::nested]
    // pub fn on_update<F: FnOnce() -> R,R>(&self, func:F) -> Option<R> {
    //     let first_call = use_state(||true);
    //     let mut recalc = false ;
    //     self.observe_with(|_| {recalc = true);
    //     if recalc {
    //         Some(func())
    //     } else {
    //         None
    //     }
    // }
}
impl<T> CloneReactiveState<T> for AtomUndo<T>
where
    T: Clone + 'static,
{
    /// returns a clone of the stored state panics if not stored.
    fn get(&self) -> T {
        clone_reactive_state_with_id::<T>(self.id).expect("state should be present")
    }

    fn soft_get(&self) -> Option<T> {
        clone_reactive_state_with_id::<T>(self.id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        reactive_state_access::{atom::Atom, atom_undo::AtomUndo, reaction::Reaction},
        *,
    };

    #[atom]
    fn c() -> Atom<i32> {
        0
    }

    // #[reaction]
    // fn count_print_when_update() -> Reaction<i32> {
    //     let c = c();
    //     let _update = a_reversible().on_update(|| {
    //         println!("UPDATE !!!");
    //         c.update(|v| *v = *v + 1)
    //     });
    //     c.get()
    // }

    #[atom(undo)]
    fn a_reversible() -> AtomUndo<i32> {
        0
    }

    #[atom(undo)]
    fn b_reversible() -> AtomUndo<i32> {
        0
    }
    #[reaction]
    fn a_b_reversible_subtraction() -> Reaction<i32> {
        let a = a_reversible().observe();
        let b = b_reversible().observe();
        a - b
    }

    #[test]
    fn test_get_with() {
        a_reversible().set(3);
        b_reversible().set(5);

        a_reversible().get_with(|v| assert_eq!(v, &3, "We should get 3"));
        b_reversible().get_with(|v| assert_eq!(v, &5, "We should get 5"));
    }

    // #[test]
    // fn test_on_update() {
    //     let print = count_print_when_update();
    //     a_reversible().update(|v| *v = 32);
    //     a_reversible().set(2);
    //     a_reversible().set(25);
    //     a_reversible().set(1);
    //
    //     println!("{:?}", print.get());
    //
    //     assert_eq!(print.get(), 5)
    // }

    #[test]
    fn test_undo() {
        a_reversible().set(3);

        a_reversible().set(5);

        b_reversible().set(10);

        a_reversible().set(4);

        assert_eq!(a_reversible().get(), 4, "We should get 4 as value for a");
        global_undo_queue().travel_backwards();
        assert_eq!(b_reversible().get(), 10, "We should get 10 as value for b");
        global_undo_queue().travel_backwards();
        assert_eq!(a_reversible().get(), 5, "We should get 5 as value for a");
        eprintln!("{:?}", a_reversible().get());
        eprintln!("{:?}", b_reversible().get());
        eprintln!("{:?}", global_undo_queue());

        global_undo_queue().travel_backwards(); // Why do we need 2 times         global_undo_queue().travel_backwards(); ?
        eprintln!("{:?}", a_reversible().get());
        eprintln!("{:?}", b_reversible().get());
        eprintln!("{:?}", global_undo_queue());
        global_undo_queue().travel_backwards();
        global_undo_queue().travel_backwards();
        assert_eq!(a_reversible().get(), 3, "We should get 3 as value for a");
        eprintln!("{:?}", a_reversible().get());
        eprintln!("{:?}", b_reversible().get());
        eprintln!("{:?}", global_undo_queue());
        global_undo_queue().travel_backwards();
        global_undo_queue().travel_backwards();
        assert_eq!(a_reversible().get(), 0, "We should get 0 as value for a");
    }

    #[test]
    fn test_update() {
        a_reversible().set(10);
        b_reversible().set(10);

        a_reversible().update(|state| *state = 45);

        assert_eq!(a_reversible().get(), 45, "We should get 45 as value for a");
    }

    #[test]
    fn test_inert_set() {
        let a_b_reversible_subtraction = a_b_reversible_subtraction();
        a_reversible().inert_set(155);
        assert_eq!(a_reversible().get(), 155, "We should get 155");
        assert_eq!(
            a_b_reversible_subtraction.get(),
            0,
            "We should get 0 since a & b are set to 0"
        );
    }

    #[test]
    fn test_delete() {
        a_reversible().delete();

        eprintln!("{:?}", a_reversible().get());

        assert_eq!(
            a_reversible().state_exists(),
            false,
            "The state  a_reversible should not exist"
        );
    }
}
