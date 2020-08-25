use crate::observable::Observable;
use crate::reactive_state_functions::*;
use crate::store::RxFunc;
use crate::store::StorageKey;
use std::marker::PhantomData;
// use seed::prelude::*;
// marker types

/// An atom is an observable and changeable piece of state.
/// You can use it to update a components and rerender specific part of the DOM
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
    /// Instantiate a new atom
    pub fn new(id: StorageKey) -> Atom<T> {
        Atom {
            id,
            _phantom_data_stored_type: PhantomData,
        }
    }

    /// Stores a value of type T in a backing Store **without** reaction for observers
    ///  ## Todo doc
    /// - add example maybe
    /// - When to use it
    pub fn inert_set(self, value: T)
    where
        T: 'static,
    {
        set_inert_atom_state_with_id(value, self.id);
    }
    /// Stores a value of type T in a backing Store **with** a reaction for observers
    ///  ```
    /// use atomic_hooks::atom;
    /// #[atom]
    ///
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
    /// #[atom]
    /// fn a() -> Atom<i32> {
    /// 0
    /// }
    ///  a().update(|state| *state = 45);
    ///  assert_eq!(a().get(), 45, "We should get 3 as value for a");
    ///
    /// ```
    ///
    /// This is use for example when we want to update a component rendering depending of a state.
    /// We update the atom so the component will rerender with the new state.
    /// If many components subscribed to the atom, then all of them will get the update.
    ///
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
    pub fn remove(self) -> Option<T> {
        remove_reactive_state_with_id(self.id)
    }
    /// ## Question :
    /// Why do we have remove and delete ?
    ///  ## Todo doc
    /// - add example maybe
    /// - When to use it
    pub fn delete(self) {
        self.remove();
    }
    ///  ## Todo doc
    /// - add example maybe
    /// - When to use it
    pub fn reset_to_default(&self) {
        (clone_reactive_state_with_id::<RxFunc>(self.id)
            .unwrap()
            .func)();
        execute_reaction_nodes(&self.id);
    }

    /// Check if the state does exists in the store
    ///  ## Todo doc
    /// - add example maybe
    /// - When to use it
    pub fn state_exists(self) -> bool {
        reactive_state_exists_for_id::<T>(self.id)
    }

    /// Get the state without reactions
    ///  ## Todo doc
    /// - add example maybe
    /// - When to use it
    pub fn get_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        read_reactive_state_with_id(self.id, func)
    }

    /// Triggers the passed function when the atom is updated
    ///  ## Todo doc
    /// - add example maybe
    /// - When to use it
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
/// An AtomUndo is similar to a regular atom except that its is reversible and is stored in a global state.
/// ```
/// use atomic_hooks_macros::*;
/// use store::RxFunc;
/// use atomic_hooks::{global_undo_queue, AtomUndo, GlobalUndo, CloneReactiveState};
///
/// #[atom(undo)]
/// fn a() -> AtomUndo<i32> {
///     0
/// }
///
/// #[atom(undo)]
///fn b() -> AtomUndo<i32> {
///    0
/// }
///
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
///
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

    // stores a value of type T in a backing Store
    pub fn inert_set(self, value: T)
    where
        T: 'static,
    {
        set_inert_atom_state_with_id_with_undo(value, self.id);
    }
    // stores a value of type T in a backing Store
    pub fn set(self, value: T)
    where
        T: 'static,
    {
        set_atom_state_with_id_with_undo(value, self.id);
    }

    pub fn update<F: FnOnce(&mut T) -> ()>(&self, func: F)
    where
        T: 'static,
    {
        update_atom_state_with_id_with_undo(self.id, func);
    }
    pub fn id(&self) -> StorageKey {
        self.id
    }

    pub fn remove(self) -> Option<T> {
        remove_reactive_state_with_id_with_undo(self.id)
    }

    pub fn delete(self) {
        self.remove();
    }

    pub fn reset_to_default(&self) {
        (clone_reactive_state_with_id::<RxFunc>(self.id)
            .unwrap()
            .func)();
        execute_reaction_nodes(&self.id);
    }

    pub fn state_exists(self) -> bool {
        reactive_state_exists_for_id::<T>(self.id)
    }

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
    pub fn new(id: StorageKey) -> Reaction<T> {
        Reaction {
            id,

            _phantom_data_stored_type: PhantomData,
        }
    }

    pub fn remove(self) -> Option<T> {
        remove_reactive_state_with_id(self.id)
    }

    pub fn delete(self) {
        self.remove();
    }

    pub fn force_trigger(&self) {
        (clone_reactive_state_with_id::<RxFunc>(self.id)
            .unwrap()
            .func)();
    }

    pub fn state_exists(self) -> bool {
        reactive_state_exists_for_id::<T>(self.id)
    }

    pub fn get_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        read_reactive_state_with_id(self.id, func)
    }

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

    #[topo::nested]
    fn has_changed(&self) -> bool {
        let previous_value_access = crate::hooks_state_functions::use_state(|| self.get());
        previous_value_access
            .get_with(|previous_value| self.observe_with(|new_value| new_value != previous_value))
    }

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

    #[topo::nested]
    fn has_changed(&self) -> bool {
        let previous_value_access = crate::hooks_state_functions::use_state(|| self.get());
        previous_value_access
            .get_with(|previous_value| self.observe_with(|new_value| new_value != previous_value))
    }

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

use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

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
    fn a() -> AtomUndo<i32> {
        0
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
        a_reversible().update(|state| *state = 45);
        assert_eq!(a_reversible().get(), 45, "We should get 3 as value for a");
    }

    // #[test]
    // fn test_update() {
    //     a_reversible().update(|state| *state = 45);
    //     assert_eq!(a_reversible().get(), 45, "We should get 3 as value for a");
    // }
}
