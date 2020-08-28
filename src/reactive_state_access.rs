use crate::{
    observable::Observable,
    reactive_state_functions::*,
    store::{RxFunc, StorageKey},
};
use std::marker::PhantomData;
// use seed::prelude::*;
// marker types

/// An atom is an observable and changeable piece of state.
/// You can use it to update a component and render specific part of the DOM.
pub struct Atom<T> {
    pub id: StorageKey,
    pub _phantom_data_stored_type: PhantomData<T>,
}

impl<T> std::fmt::Debug for Atom<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:#?})", self.id)
    }
}

impl<T> Clone for Atom<T> {
    fn clone(&self) -> Atom<T> {
        Atom::<T> {
            id: self.id, // is is not supposed to be unique ?

            _phantom_data_stored_type: PhantomData::<T>,
        }
    }
}

impl<T> Copy for Atom<T> {}

impl<T> Atom<T>
where
    T: 'static,
{
    /// Instantiate a new atom.
    pub fn new(id: StorageKey) -> Atom<T> {
        Atom {
            id,
            _phantom_data_stored_type: PhantomData,
        }
    }

    /// Stores a value of type T in a backing Store **without** reaction for
    /// observers.
    ///
    /// ```
    /// use atomic_hooks::{Atom, CloneReactiveState, Observable};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    /// fn b() -> Atom<i32> {
    ///     0
    /// }
    /// #[reaction]
    /// fn reaction_a_b_subtraction() {
    ///     let a = a().observe();
    ///     let b = b().observe();
    ///     (a - b)
    /// }
    /// a().inert_set(1);
    /// let diff = reaction_a_b_subtraction();
    /// assert_eq!(a().get(), 1);
    /// assert_eq!(
    ///     diff.get(),
    ///     0,
    ///     "We should still get 0 since we use inert setting"
    /// );
    /// ```
    ///   ## Todo doc
    /// - Add a description that explains relevant use case for this method
    pub fn inert_set(self, value: T)
    where
        T: 'static,
    {
        set_inert_atom_state_with_id(value, self.id);
    }
    /// Stores a value of type T in a backing Store **with** a reaction for
    /// observers.  
    ///
    /// ```
    /// use atomic_hooks::{Atom, CloneReactiveState};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    /// 0
    /// }
    ///
    /// a().set(1);
    ///
    /// assert_eq!(a().get(), 1);
    ///
    ///  ```
    /// - add example maybe
    /// - When to use it
    pub fn set(self, value: T)
    where
        T: 'static,
    {
        set_atom_state_with_id(value, self.id);
    }

    /// Pass a function that update the atom state related
    /// This update will trigger reactions and observers will get the update
    /// ```
    /// use atomic_hooks::{Atom, CloneReactiveState};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    /// a().update(|state| *state = 45);
    /// assert_eq!(a().get(), 45, "We should get 45 as value for a");
    /// ```
    ///
    /// This is use for example when we want to update a component rendering
    /// depending of a state. We update the atom so the component will
    /// rerender with the new state. If many components subscribed to the
    /// atom, then all of them will get the update.
    pub fn update<F: FnOnce(&mut T) -> ()>(&self, func: F)
    where
        T: 'static,
    {
        update_atom_state_with_id(self.id, func);
    }
    pub fn id(&self) -> StorageKey {
        self.id
    }

    /// Use to remove an atom from the global state
    /// ```
    /// use atomic_hooks::Atom;
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// a().remove();
    ///
    /// assert_eq!(a().state_exists(), false, "The a state should not exist");
    /// ```
    pub fn remove(self) -> Option<T> {
        remove_reactive_state_with_id(self.id)
    }
    /// ## Question :
    /// Why do we have remove and delete ?
    ///  
    /// ```
    /// use atomic_hooks::Atom;
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// a().delete();
    ///
    /// assert_eq!(a().state_exists(), false, "The a state should not exist");
    /// ```
    /// ## Todo doc
    /// - When to use it
    pub fn delete(self) {
        self.remove();
    }
    /// Reset to the initial value
    /// ```
    /// use atomic_hooks::{Atom, CloneReactiveState};
    /// #[atom]
    /// fn a() -> Atom<i32> {
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

    /// Check if the state does exist in the store.
    /// ```
    /// use atomic_hooks::Atom;
    /// #[atom]
    /// fn a() -> Atom<i32> {
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
    /// use atomic_hooks::Atom;
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    /// a().set(3);
    ///
    /// a().get_with(|v| assert_eq!(v, &3, "We should get 3"));
    /// ```
    ///  ## Todo doc
    /// - When to use it
    pub fn get_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        read_reactive_state_with_id(self.id, func)
    }

    /// Triggers the passed function when the atom is updated
    /// This method needs to be use inside a function body that has the
    /// attributes **[reaction]**.
    ///
    /// ```
    /// #[reaction]
    /// fn count_print_when_update() -> Reaction<i32> {
    ///     let c = c();
    ///     let update = a().on_update(|| {
    ///         println!("UPDATE !!!");
    ///         c.update(|mut v| *v = *v + 1)
    ///     });
    ///     c.get()
    /// }
    ///
    /// #[test]
    /// fn test_on_update() {
    ///     let print = count_print_when_update();
    ///     a().update(|v| *v = 32);
    ///     a().set(2);
    ///     a().set(25);
    ///     a().set(1);
    ///
    ///     println!("{:?}", print.get());
    ///
    ///     assert_eq!(print.get(), 5)
    /// }
    /// ```
    ///
    ///  ## Todo doc
    /// - When to use it
    /// - Improve the doc
    pub fn on_update<F: FnOnce() -> R, R>(&self, func: F) -> Option<R> {
        let mut recalc = false;
        self.observe_with(|_| recalc = true);
        if recalc {
            Some(func())
        } else {
            None
        }
    }
}

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

/// A reaction is an observable state combined from one or multiple atom state.
/// Literally you can write code that is function of atom state value to produce
/// a new value which you can observe. The new value will get automatically
/// updated as long as the update on the atom is not **inert**.  
///
/// ```
/// use atomic_hooks::{Atom, CloneReactiveState, Observable, Reaction};
///
/// #[atom]
/// fn a() -> Atom<i32> {
///     0
/// }
///
/// #[atom]
/// fn b() -> Atom<i32> {
///     0
/// }
///
/// #[reaction]
/// fn a_b_subtraction() -> Reaction<i32> {
///     let a = a().observe();
///     let b = b().observe();
///     (a - b)
/// }
/// // we have the state
/// // of a - b and we can get it when never we want.
/// // the value should always be automatically updated
/// fn test_reaction() {
///     let a_b_subtraction = a_b_subtraction();
///
///     a().set(0);
///     b().set(0);
///     a().update(|state| *state = 40);
///     assert_eq!(a().get(), 40, "We should get 40 as value for a");
///     assert_eq!(
///         a_b_subtraction.get(),
///         40,
///         "We should get 40 for subtraction because setting"
///     );
///
///     b().set(10);
///     assert_eq!(
///         a_b_subtraction.get(),
///         30,
///         "We should get 40 for subtraction because setting"
///     );
///     b().inert_set(0);
///     assert_eq!(
///         a_b_subtraction.get(),
///         30,
///         "We should get 30 for subtraction because setting inert"
///     );
///     b().set(20);
///     assert_eq!(
///         a_b_subtraction.get(),
///         20,
///         "We should get 20 for subtraction because setting"
///     );
/// }
/// ```
pub struct Reaction<T> {
    pub id: StorageKey,

    pub _phantom_data_stored_type: PhantomData<T>,
}

impl<T> std::fmt::Debug for Reaction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:#?})", self.id)
    }
}
//

//
impl<T> Clone for Reaction<T> {
    fn clone(&self) -> Reaction<T> {
        Reaction::<T> {
            id: self.id,

            _phantom_data_stored_type: PhantomData::<T>,
        }
    }
}

impl<T> Copy for Reaction<T> {}

impl<T> Reaction<T>
where
    T: 'static,
{
    /// Create a new reaction
    pub fn new(id: StorageKey) -> Reaction<T> {
        Reaction {
            id,

            _phantom_data_stored_type: PhantomData,
        }
    }
    /// Remove the reaction from the global state
    /// ```
    /// use atomic_hooks::{Atom, Observable, Reaction};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[atom]
    /// fn b() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[reaction]
    /// fn a_b_subtraction() -> Reaction<i32> {
    ///     let a = a().observe();
    ///     let b = b().observe();
    ///     (a - b)
    /// }
    /// fn test_delete() {
    ///     let a_b_subtraction = a_b_subtraction();
    ///     a_b_subtraction.remove();
    ///
    ///     assert_eq!(
    ///         a_b_subtraction.state_exists(),
    ///         false,
    ///         "The state has been removed, so it should not exist"
    ///     );
    /// }
    /// ```
    pub fn remove(self) -> Option<T> {
        remove_reactive_state_with_id(self.id)
    }

    ///
    /// ```
    /// use atomic_hooks::{Atom, Observable, Reaction};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[atom]
    /// fn b() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[reaction]
    /// fn a_b_subtraction() -> Reaction<i32> {
    ///     let a = a().observe();
    ///     let b = b().observe();
    ///     (a - b)
    /// }
    /// fn test_delete() {
    ///     let a_b_subtraction = a_b_subtraction();
    ///     a_b_subtraction.delete();
    ///
    ///     assert_eq!(
    ///         a_b_subtraction.state_exists(),
    ///         false,
    ///         "The state has been removed, so it should not exist"
    ///     );
    /// }
    /// ```
    pub fn delete(self) {
        self.remove();
    }

    /// This method force the update of the new combined value
    /// ## Question
    /// - I thought the new value was updated automatically, isn't ?
    /// - When & why to use this method ?
    pub fn force_trigger(&self) {
        (clone_reactive_state_with_id::<RxFunc>(self.id)
            .unwrap()
            .func)();
    }
    /// Check if the state_exist
    /// ```
    /// use atomic_hooks::{Atom, Observable, Reaction};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[atom]
    /// fn b() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[reaction]
    /// fn a_b_subtraction() -> Reaction<i32> {
    ///     let a = a().observe();
    ///     let b = b().observe();
    ///     (a - b)
    /// }
    /// fn test_state_exist() {
    ///     let a_b_subtraction = a_b_subtraction();
    ///
    ///     assert_eq!(
    ///         a_b_subtraction.state_exists(),
    ///         true,
    ///         "The state should exist"
    ///     );
    ///     a_b_subtraction.delete();
    ///
    ///     assert_eq!(
    ///         a_b_subtraction.state_exists(),
    ///         false,
    ///         "The state has been removed, so it should not exist"
    ///     );
    /// }
    /// ```
    pub fn state_exists(self) -> bool {
        reactive_state_exists_for_id::<T>(self.id)
    }
    /// Let you get the value as a reference from a closure.
    /// ```
    /// use atomic_hooks::{Atom, Observable, Reaction};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[atom]
    /// fn b() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[reaction]
    /// fn a_b_subtraction() -> Reaction<i32> {
    ///     let a = a().observe();
    ///     let b = b().observe();
    ///     (a - b)
    /// }
    ///
    /// fn test_get_with() {
    ///     let a_b_subtraction = a_b_subtraction();
    ///     a_b_subtraction.get_with(|v| assert_eq!(v, &0, "We should get 0"));
    ///     a().set(10);
    ///     a_b_subtraction.get_with(|v| assert_eq!(v, &10, "We should get 10"));
    /// }
    /// ```
    pub fn get_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        read_reactive_state_with_id(self.id, func)
    }
    /// Triggers the passed function when the atom is updated
    /// This method needs to be use inside a function body that has the
    /// attributes **[reaction]**.
    /// ```
    /// use atomic_hooks::{Atom, CloneReactiveState, Observable, Reaction};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[atom]
    /// fn b() -> Atom<i32> {
    ///     0
    /// }
    /// #[atom]
    /// fn c() -> Atom<i32> {
    ///     0
    /// }
    /// #[reaction]
    /// fn a_b_subtraction() -> Reaction<i32> {
    ///     let a = a().observe();
    ///     let b = b().observe();
    ///     (a - b)
    /// }
    /// #[reaction]
    /// fn count_subtraction_when_update() -> Reaction<i32> {
    ///     let c = c();
    ///     let update = a_b_subtraction().on_update(|| {
    ///         println!("UPDATE !!!");
    ///         c.update(|mut v| *v = *v + 1)
    ///     });
    ///     c.get()
    /// }
    /// fn test_on_update() {
    ///     let count = count_subtraction_when_update();
    ///     a().update(|v| *v = 32);
    ///     a().set(2);
    ///     a().set(25);
    ///     a().set(1);
    ///     assert_eq!(count.get(), 5);
    /// }
    /// ```
    #[topo::nested]
    pub fn on_update<F: FnOnce() -> R, R>(&self, func: F) -> Option<R> {
        let first_call_accessor = crate::hooks_state_functions::use_state(|| true);
        let mut recalc = false;

        self.observe_with(|_| {
            if first_call_accessor.get() {
                first_call_accessor.set(false)
            } else {
                recalc = true
            }
        });
        if recalc {
            Some(func())
        } else {
            None
        }
    }
    /// This method give us the possibility to know if a reaction has been
    /// updated.
    /// ```
    /// use atomic_hooks::{Atom, CloneReactiveState, Observable, Reaction};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[atom]
    /// fn b() -> Atom<i32> {
    ///     0
    /// }
    /// #[reaction]
    /// fn a_b_subtraction() -> Reaction<i32> {
    ///     let a = a().observe();
    ///     let b = b().observe();
    ///     (a - b)
    /// }
    ///
    /// #[reaction]
    /// fn count_subtraction_when_update() -> Reaction<i32> {
    ///     let c = c();
    ///     let update = a_b_subtraction().on_update(|| {
    ///         println!("UPDATE !!!");
    ///         c.update(|mut v| *v = *v + 1)
    ///     });
    ///     c.get()
    /// }
    /// fn test_has_updated() {
    ///     let count = count_subtraction_when_update();
    ///     a().update(|v| *v = 32);
    ///     assert_eq!(count.has_updated(), true);
    ///     a().set(32);
    ///     assert_eq!(
    ///         count.has_updated(),
    ///         false,
    ///         "No update should have been done since we updated a to 32 which is the same value as \
    ///              before"
    ///     );
    ///     a().set(25);
    ///     assert_eq!(count.has_updated(), true);
    ///     a().set(1);
    ///     assert_eq!(count.has_updated(), true);
    /// }
    /// ```
    #[topo::nested]
    pub fn has_updated(&self) -> bool {
        let first_call_accessor = crate::hooks_state_functions::use_state(|| true);
        let mut recalc = false;

        self.observe_with(|_| {
            if first_call_accessor.get() {
                first_call_accessor.set(false)
            } else {
                recalc = true
            }
        });
        recalc
    }
}

// If the stored type is clone, then implement clone for ReactiveStateAccess
pub trait CloneReactiveState<T>
where
    T: Clone + 'static,
{
    fn get(&self) -> T;
    fn soft_get(&self) -> Option<T>;
}

pub trait ObserveChangeReactiveState<T>
where
    T: Clone + 'static + PartialEq,
{
    fn observe_change(&self) -> (Option<T>, T);
    fn has_changed(&self) -> bool;
    fn on_change<F: FnOnce(&T, &T) -> R, R>(&self, func: F) -> R;
}

use crate::state_access::CloneState;

// The below is broke as need None if no prior state
impl<T> ObserveChangeReactiveState<T> for Atom<T>
where
    T: Clone + 'static + PartialEq,
{
    /// Let you get the last changes on an Atom state
    ///
    /// ## Todo
    /// - Improve the name of the method, because user might be expecting having
    ///   an observable while in fact the value from this method does not update
    ///   change over time but give only the last change.
    /// - the unit is failling for this method because option gives always None
    ///   as value.
    /// ```
    /// use atomic_hooks::{Atom, ObserveChangeReactiveState};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[test]
    /// fn test_observe_on_atom() {
    ///     let a = a();
    ///     let change = a.observe_change();
    ///     assert_eq!(change.0.is_none(), true);
    ///     assert_eq!(change.1, 0);
    ///     a.set(1);
    ///     let change2 = a.observe_change();
    ///     assert_eq!(change2.0.unwrap(), 0);
    ///     assert_eq!(change2.1, 1);
    /// }
    /// ```
    #[topo::nested]
    fn observe_change(&self) -> (Option<T>, T) {
        let previous_value_access = crate::hooks_state_functions::use_state(|| self.get());
        previous_value_access.get_with(|previous_value| {
            self.observe_with(|new_value| {
                if *previous_value != *new_value {
                    previous_value_access.set(new_value.clone());
                    (Some(previous_value.clone()), new_value.clone())
                } else {
                    (None, new_value.clone())
                }
            })
        })
    }
    /// This method gives us the possibility to know if the state of an atom has
    /// been changed.
    ///
    /// ## Todo
    /// - the unit test is failling for this method
    ///
    /// ```
    /// use atomic_hooks::{Atom, ObserveChangeReactiveState};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[test]
    /// fn test_has_changed_on_atom() {
    ///     let a = a();
    ///     a.set(1);
    ///
    ///     assert_eq!(a.has_changed(), true);
    ///     a.set(1);
    ///     assert_eq!(a.has_changed(), false);
    /// }
    /// ```
    #[topo::nested]
    fn has_changed(&self) -> bool {
        let previous_value_access = crate::hooks_state_functions::use_state(|| self.get());
        previous_value_access
            .get_with(|previous_value| self.observe_with(|new_value| new_value != previous_value))
    }
    /// This method gives the opportunity to trigger a function and use the
    /// values from the changes.
    ///
    /// ## Todo
    /// - the unit test is failling for this method after a.set(2)
    ///
    /// ```
    /// use atomic_hooks::{
    ///     Atom, CloneReactiveState, Observable, ObserveChangeReactiveState, Reaction,
    /// };
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[test]
    /// fn test_on_changes_on_atom() {
    ///     let a = a();
    ///     let mut previous = 99;
    ///     let mut current = 99;
    ///     a.on_change(|p, c| {
    ///         previous = *p;
    ///         current = *c;
    ///     });
    ///     assert_eq!(previous, 0); //todo : should we expect None when init ?
    ///     assert_eq!(current, 0);
    ///     a.set(1);
    ///     a.on_change(|p, c| {
    ///         previous = *p;
    ///         current = *c;
    ///     });
    ///     assert_eq!(previous, 0);
    ///     assert_eq!(current, 1);
    ///     a.set(1);
    ///     a.on_change(|p, c| {
    ///         previous = *p;
    ///         current = *c;
    ///     });
    ///     assert_eq!(previous, 0);
    ///     assert_eq!(current, 1);
    ///     a.set(2);
    ///     a.on_change(|p, c| {
    ///         previous = *p;
    ///         current = *c;
    ///     });
    ///     assert_eq!(previous, 1, "we should get 1");
    ///     assert_eq!(current, 2, "we should get 2");
    /// }
    /// ```
    fn on_change<F: FnOnce(&T, &T) -> R, R>(&self, func: F) -> R {
        let previous_value_access = crate::hooks_state_functions::use_state(|| self.get());
        previous_value_access.get_with(|previous_value| {
            self.observe_with(|new_value| func(previous_value, new_value))
        })
    }
}

impl<T> ObserveChangeReactiveState<T> for Reaction<T>
where
    T: Clone + 'static + PartialEq,
{
    /// Let you get the last changes on a reaction.
    ///
    /// ## Todo
    ///
    /// - the unit test is failing so I guess we need to investigate the bug
    /// ```
    /// use atomic_hooks::{Atom, Observable, ObserveChangeReactiveState, Reaction};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[atom]
    /// fn b() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[reaction]
    /// fn a_b_subtraction() -> Reaction<i32> {
    ///     let a = a().observe();
    ///     let b = b().observe();
    ///     (a - b)
    /// }
    /// #[test]
    /// fn test_observe_changes_on_reaction() {
    ///     let a_b_subtraction = a_b_subtraction();
    ///     let changes = a_b_subtraction.observe_change();
    ///     assert_eq!(changes.0.is_none(), true);
    ///     assert_eq!(changes.1, 0);
    ///
    ///     a().set(2);
    ///     let changes = a_b_subtraction.observe_change();
    ///     assert_eq!(changes.0.unwrap(), 1);
    ///     assert_eq!(changes.1, 2);
    /// }
    /// ```
    #[topo::nested]
    fn observe_change(&self) -> (Option<T>, T) {
        let previous_value_access = crate::hooks_state_functions::use_state(|| self.get());
        previous_value_access.get_with(|previous_value| {
            self.observe_with(|new_value| {
                if *previous_value != *new_value {
                    previous_value_access.set(new_value.clone());
                    (Some(previous_value.clone()), new_value.clone())
                } else {
                    (None, new_value.clone())
                }
            })
        })
    }
    /// Let you know if changes has been made.
    ///
    /// ## Todo
    /// - the unit test is failing so I guess we need to investigate the bug
    /// ```
    /// use atomic_hooks::{Atom, Observable, ObserveChangeReactiveState, Reaction};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[atom]
    /// fn b() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[reaction]
    /// fn a_b_subtraction() -> Reaction<i32> {
    ///     let a = a().observe();
    ///     let b = b().observe();
    ///     (a - b)
    /// }
    /// #[test]
    /// fn test_has_changes_on_reaction() {
    ///     let a_b_subtraction = a_b_subtraction();
    ///
    ///     a().set(2);
    ///     let changes_happened = a_b_subtraction.has_changed();
    ///     assert_eq!(changes_happened, true);
    ///
    ///     a().set(3);
    ///     let changes_happened = a_b_subtraction.has_changed();
    ///     assert_eq!(changes_happened, true);
    ///
    ///     a().set(3);
    ///     let changes_happened = a_b_subtraction.has_changed();
    ///     assert_eq!(changes_happened, false);
    /// }
    /// ```
    #[topo::nested]
    fn has_changed(&self) -> bool {
        let previous_value_access = crate::hooks_state_functions::use_state(|| self.get());
        previous_value_access
            .get_with(|previous_value| self.observe_with(|new_value| new_value != previous_value))
    }

    /// Let you apply a function on previous and current value from changes.
    ///
    /// ## Todo
    ///
    /// - the unit test is failing so I guess we need to investigate the bug
    /// ```
    /// use atomic_hooks::{Atom, Observable, ObserveChangeReactiveState, Reaction};
    /// #[atom]
    /// fn a() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[atom]
    /// fn b() -> Atom<i32> {
    ///     0
    /// }
    ///
    /// #[reaction]
    /// fn a_b_subtraction() -> Reaction<i32> {
    ///     let a = a().observe();
    ///     let b = b().observe();
    ///     (a - b)
    /// }
    /// #[test]
    /// fn test_on_changes_on_reaction() {
    ///     let a_b_subtraction = a_b_subtraction();
    ///     let mut previous = 99;
    ///     let mut current = 99;
    ///     a_b_subtraction.on_change(|p, c| {
    ///         previous = *p;
    ///         current = *c;
    ///     });
    ///     assert_eq!(previous, 0);
    ///     assert_eq!(current, 0);
    ///     a().set(1);
    ///     a_b_subtraction.on_change(|p, c| {
    ///         previous = *p;
    ///         current = *c;
    ///     });
    ///     assert_eq!(previous, 0);
    ///     assert_eq!(current, 1);
    ///     a().set(2);
    ///     a_b_subtraction.on_change(|p, c| {
    ///         previous = *p;
    ///         current = *c;
    ///     });
    ///     assert_eq!(previous, 1); /
    ///     assert_eq!(current, 2);
    /// ```
    fn on_change<F: FnOnce(&T, &T) -> R, R>(&self, func: F) -> R {
        let previous_value_access = crate::hooks_state_functions::use_state(|| self.get());
        previous_value_access.get_with(|previous_value| {
            self.observe_with(|new_value| func(previous_value, new_value))
        })
    }
}

impl<T> CloneReactiveState<T> for Atom<T>
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

impl<T> CloneReactiveState<T> for Reaction<T>
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

// If the underlying type provides display then so does the ReactiveStateAccess
impl<T> std::fmt::Display for Atom<T>
where
    T: std::fmt::Display + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_with(|t| write!(f, "{}", t))
    }
}

use std::ops::{Add, Div, Mul, Sub};

impl<T> Add for Atom<T>
where
    T: Copy + Add<Output = T> + 'static,
{
    type Output = T;

    fn add(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o + *s))
    }
}

impl<T> Mul for Atom<T>
where
    T: Copy + Mul<Output = T> + 'static,
{
    type Output = T;

    fn mul(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o * *s))
    }
}

impl<T> Div for Atom<T>
where
    T: Copy + Div<Output = T> + 'static,
{
    type Output = T;

    fn div(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o / *s))
    }
}

impl<T> Sub for Atom<T>
where
    T: Copy + Sub<Output = T> + 'static,
{
    type Output = T;

    fn sub(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o - *s))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use atomic_hooks_macros::*;
    use store::RxFunc;

    #[atom]
    fn a() -> Atom<i32> {
        0
    }

    #[atom]
    fn b() -> Atom<i32> {
        0
    }

    #[reaction]
    fn a_b_subtraction() -> Reaction<i32> {
        let a = a().observe();
        let b = b().observe();
        (a - b)
    }

    #[reaction]
    fn a_b_reversible_subtraction() -> Reaction<i32> {
        let a = a_reversible().observe();
        let b = b_reversible().observe();
        (a - b)
    }

    #[atom]
    fn c() -> Atom<i32> {
        0
    }

    #[reaction]
    fn count_print_when_update() -> Reaction<i32> {
        let c = c();
        let update = a().on_update(|| {
            println!("UPDATE !!!");
            c.update(|mut v| *v = *v + 1)
        });
        c.get()
    }

    #[reaction]
    fn count_subtraction_when_update() -> Reaction<i32> {
        let c = c();
        let update = a_b_subtraction().on_update(|| {
            println!("UPDATE !!!");
            c.update(|mut v| *v = *v + 1)
        });
        c.get()
    }

    #[atom(undo)]
    fn a_reversible() -> AtomUndo<i32> {
        0
    }

    #[atom(undo)]
    fn b_reversible() -> AtomUndo<i32> {
        0
    }
    #[test]
    fn test_on_changes_on_reaction() {
        let a_b_subtraction = a_b_subtraction();
        let mut previous = 99;
        let mut current = 99;
        a_b_subtraction.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 0); //todo : should we expect None when init ?
        assert_eq!(current, 0);
        a().set(1);
        a_b_subtraction.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 0); //todo : should we expect None when init ?
        assert_eq!(current, 1);
        a().set(2);
        a_b_subtraction.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 1); //todo : should we expect None when init ?
        assert_eq!(current, 2);
    }
    #[test]
    fn test_has_changes_on_reaction() {
        let a_b_subtraction = a_b_subtraction();

        a().set(2);
        let changes_happened = a_b_subtraction.has_changed();
        assert_eq!(changes_happened, true);

        a().set(3);
        let changes_happened = a_b_subtraction.has_changed();
        assert_eq!(changes_happened, true);

        a().set(3);
        let changes_happened = a_b_subtraction.has_changed();
        assert_eq!(changes_happened, false);
    }
    #[test]
    fn test_observe_changes_on_reaction() {
        let a_b_subtraction = a_b_subtraction();
        let changes = a_b_subtraction.observe_change();
        assert_eq!(changes.0.is_none(), true);
        assert_eq!(changes.1, 0);

        a().set(2);
        let changes = a_b_subtraction.observe_change();
        assert_eq!(changes.0.unwrap(), 1);
        assert_eq!(changes.1, 2);
    }

    #[test]
    fn test_observe_on_atom() {
        let a = a();
        let change = a.observe_change();
        println!("{:?}", change.0);
        println!("{:?}", change.1);
        assert_eq!(change.0.is_none(), true);
        assert_eq!(change.1, 0);
        a.set(1);
        let change2 = a.observe_change();
        println!("{:?}", change2.0);
        println!("{:?}", change2.1);
        assert_eq!(change2.0.unwrap(), 0);
        assert_eq!(change2.1, 1);
    }
    #[test]
    fn test_has_changed_on_atom() {
        let a = a();
        a.set(1);

        assert_eq!(a.has_changed(), true);
        a.set(1);
        assert_eq!(a.has_changed(), false);
    }
    #[test]
    fn test_on_changes_on_atom() {
        let a = a();
        let mut previous = 99;
        let mut current = 99;
        a.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 0); //todo : should we expect None when init ?
        assert_eq!(current, 0);
        a.set(1);
        a.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 0);
        assert_eq!(current, 1);
        a.set(1);
        a.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 0);
        assert_eq!(current, 1);
        a.set(2);
        a.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 1, "we should get 1");
        assert_eq!(current, 2, "we should get 2");
    }

    #[test]
    fn test_get_with() {
        let a_b_reversible_subtraction = a_b_reversible_subtraction();
        a_reversible().set(3);
        b_reversible().set(5);

        a_reversible().get_with(|v| assert_eq!(v, &3, "We should get 3"));
        b_reversible().get_with(|v| assert_eq!(v, &5, "We should get 5"));
        a_b_reversible_subtraction.get_with(|v| assert_eq!(v, &-2, "We should get -2"));
        a().set(3);
        b().set(5);

        a().get_with(|v| assert_eq!(v, &3, "We should get 3"));
        b().get_with(|v| assert_eq!(v, &5, "We should get 5"));
    }

    #[test]
    fn test_on_update() {
        let print = count_print_when_update();
        a().update(|v| *v = 32);
        a().set(2);
        a().set(25);
        a().set(1);

        println!("{:?}", print.get());

        assert_eq!(print.get(), 5)
    }

    #[test]
    fn test_on_update_reaction() {
        let count = count_subtraction_when_update();
        println!("{:?}", a_b_subtraction().get());
        a().update(|v| *v = 32);
        println!("{:?}", a_b_subtraction().get());
        a().set(2);
        println!("{:?}", a_b_subtraction().get());
        a().set(25);
        println!("{:?}", a_b_subtraction().get());
        a().set(1);
        println!("{:?}", a_b_subtraction().get());

        println!("{:?}", count.get());

        assert_eq!(count.get(), 5);
    }

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

        a().update(|state| *state = 40);
        assert_eq!(a().get(), 40, "We should get 40 as value for a");
    }

    #[test]
    fn test_inert_set() {
        a_reversible().inert_set(155);
        assert_eq!(a_reversible().get(), 155, "We should get 155");

        let a_b_subtraction = a_b_subtraction();
        a().set(0);
        b().set(0);

        a().inert_set(165);
        assert_eq!(
            a_b_subtraction.get(),
            0,
            "We should get 0 for subtraction because inert setting"
        );
        assert_eq!(a().get(), 165, "We should get 165");
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

        a().delete();

        eprintln!("{:?}", a().get());

        assert_eq!(a().state_exists(), false, "The a state should not exist");
    }

    #[test]
    fn test_reaction() {
        let a_b_subtraction = a_b_subtraction();
        a().set(0);
        b().set(0);
        a().update(|state| *state = 40);
        assert_eq!(a().get(), 40, "We should get 40 as value for a");
        assert_eq!(
            a_b_subtraction.get(),
            40,
            "We should get 40 for subtraction because setting"
        );

        b().set(10);
        assert_eq!(
            a_b_subtraction.get(),
            30,
            "We should get 40 for subtraction because setting"
        );
        b().inert_set(0);
        assert_eq!(
            a_b_subtraction.get(),
            30,
            "We should get 30 for subtraction because setting inert"
        );
        b().set(20);
        assert_eq!(
            a_b_subtraction.get(),
            20,
            "We should get 20 for subtraction because setting"
        );
    }

    #[test]
    fn test_reversible_reaction() {
        let a_b_reversible_subtraction = a_b_reversible_subtraction();
        a_reversible().set(0);
        b_reversible().set(0);
        a_reversible().update(|state| *state = 40);
        assert_eq!(a_reversible().get(), 40, "We should get 40 as value for a");
        assert_eq!(
            a_b_reversible_subtraction.get(),
            40,
            "We should get 40 for subtraction because setting"
        );

        global_undo_queue().travel_backwards();
        assert_eq!(
            a_b_reversible_subtraction.get(),
            0,
            "We should get 0 because back in time"
        );

        b_reversible().inert_set(0);
        assert_eq!(
            a_b_reversible_subtraction.get(),
            0,
            "We should get 0 for subtraction because setting inert"
        );
        a_reversible().set(20);
        assert_eq!(
            a_b_reversible_subtraction.get(),
            20,
            "We should get 20 for subtraction because setting"
        );
    }
}
