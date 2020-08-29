use crate::reactive_state_access::CloneReactiveState;
use crate::{
    clone_reactive_state_with_id, reactive_state_exists_for_id,
    reactive_state_functions::{
        execute_reaction_nodes, remove_reactive_state_with_id_with_undo,
        set_atom_state_with_id_with_undo, update_atom_state_with_id_with_undo,
    },
    read_reactive_state_with_id, set_inert_atom_state_with_id_with_undo,
    store::StorageKey,
    RxFunc,
};
use std::marker::PhantomData;

///
/// An AtomUndo is similar to a regular atom except that it is reversible and
/// is stored in a global state.
/// ```
/// use atomic_hooks_macros::*;
/// use store::RxFunc;
/// use atomic_hooks::{global_undo_queue, AtomUndo, GlobalUndo,
/// CloneReactiveState};
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
    /// use atomic_hooks::{AtomUndo, CloneReactiveState, Observable};
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
    /// fn reaction_a_b_subtraction() {
    ///     let a = a().observe();
    ///     let b = b().observe();
    ///     (a - b)
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
    /// use atomic_hooks::{AtomUndo, CloneReactiveState};
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
    /// use atomic_hooks::{AtomUndo, CloneReactiveState};
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
    pub fn id(&self) -> StorageKey {
        self.id
    }
    /// ```
    /// use atomic_hooks::AtomUndo;
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
    /// use atomic_hooks::AtomUndo;
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
    /// use atomic_hooks::AtomUndo;
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
    /// use atomic_hooks::AtomUndo;
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
    /// use atomic_hooks::AtomUndo;
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
