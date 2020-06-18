use crate::atomic_state_functions::*;
use std::marker::PhantomData;

///  Accessor struct that provides access to getting and setting the
///  state of the stored type
///
// #[derive(Debug)]
pub struct AtomicStateAccess<T> {
    pub id: String,
    pub _phantom_data: PhantomData<T>,
}

impl<T> std::fmt::Debug for AtomicStateAccess<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:#?})", self.id)
    }
}


impl<T> Clone for AtomicStateAccess<T> {
    fn clone(&self) -> AtomicStateAccess<T> {
        AtomicStateAccess::<T> {
            id: self.id.clone(),
            _phantom_data: PhantomData::<T>,
        }
    }
}

impl<T> AtomicStateAccess<T>
where
    T: 'static,
{
    pub fn new(id: &str) -> AtomicStateAccess<T> {
        AtomicStateAccess {
            id: id.to_string(),
            _phantom_data: PhantomData,
        }
    }

    // stores a value of type T in a backing Store
    pub fn set(self, value: T) {
        set_atomic_state_with_id(value, &self.id);
    }

    pub fn remove(self) -> Option<T> {
        remove_atomic_state_with_id(&self.id)
    }

    pub fn delete(self) {
        self.remove();
    }


    /// updates the stored state in place
    /// using the provided function
    pub fn update<F: FnOnce(&mut T) -> ()>(&self, func: F) {
        update_atomic_state_with_id(&self.id, func);
    }

    pub fn state_exists(self) -> bool {
        atomic_state_exists_for_id::<T>(&self.id)
    }

    pub fn get_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        read_atomic_state_with_id(&self.id, func)
    }
}

pub trait CloneAtomicState<T>
where
    T: Clone + 'static,
{
    fn get(&self) -> T;

    fn soft_get(&self) -> Option<T>;
}

impl<T> CloneAtomicState<T> for AtomicStateAccess<T>
where
    T: Clone + 'static,
{
    /// returns a clone of the stored state panics if not stored.
    fn get(&self) -> T {
        clone_atomic_state_with_id::<T>(&self.id).expect("state should be present")
    }

    fn soft_get(&self) -> Option<T> {
        clone_atomic_state_with_id::<T>(&self.id)
    }
}

#[derive(Clone)]
struct ChangedWrapper<T>(T);

pub trait ChangedAtomicState {
    fn changed(&self) -> bool;
}

impl<T> ChangedAtomicState for AtomicStateAccess<T>
where
    T: Clone + 'static + PartialEq,
{
    fn changed(&self) -> bool {
        if let Some(old_state) = clone_atomic_state_with_id::<ChangedWrapper<T>>(&self.id) {
            old_state.0 != self.get()
        } else {
            set_atomic_state_with_id(ChangedWrapper(self.get()), &self.id);
            true
        }
    }
}

impl<T> std::fmt::Display for AtomicStateAccess<T>
where
    T: std::fmt::Display + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = self.get_with(|t| format!("{}", t));
        write!(f, "{}", val)
    }
}

use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

impl<T> Add for AtomicStateAccess<T>
where
    T: Copy + Add<Output = T> + 'static,
{
    type Output = T;

    fn add(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o + *s))
    }
}

impl<T> Mul for AtomicStateAccess<T>
where
    T: Copy + Mul<Output = T> + 'static,
{
    type Output = T;

    fn mul(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o * *s))
    }
}

impl<T> Div for AtomicStateAccess<T>
where
    T: Copy + Div<Output = T> + 'static,
{
    type Output = T;

    fn div(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o / *s))
    }
}

impl<T> Sub for AtomicStateAccess<T>
where
    T: Copy + Sub<Output = T> + 'static,
{
    type Output = T;

    fn sub(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o - *s))
    }
}
