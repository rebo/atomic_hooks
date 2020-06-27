use crate::atom_state_functions::*;
use std::marker::PhantomData;


// marker types
pub enum AllowUndo{}
pub enum NoUndo{}
pub  enum IsAnAtomState{}
pub  enum IsAComputedState{}

///  Accessor struct that provides access to getting and setting the
///  state of the stored type
///
// #[derive(Debug)]
pub struct AtomStateAccess<T,U,A> {
    pub id: String,
    pub inverse_fn: Option<fn(T)->()>,
    
    pub _phantom_data_stored_type: PhantomData<T>,
    pub _phantom_data_undo : PhantomData<U>,
    pub _phantom_data_accessor_type : PhantomData<A>,
}

impl<T,U,A> std::fmt::Debug for AtomStateAccess<T,U,A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:#?})", self.id)
    }
}

impl<T,U,A> Clone for AtomStateAccess<T,U,A> {
    fn clone(&self) -> AtomStateAccess<T,U,A> {
        AtomStateAccess::<T,U,A> {
            id: self.id.clone(),
            inverse_fn: self.inverse_fn.clone(),
            _phantom_data_stored_type: PhantomData::<T>,
            _phantom_data_undo : PhantomData::<U>,
            _phantom_data_accessor_type : PhantomData::<A>,
        }
    }
}

impl<T,U,A> AtomStateAccess<T,U,A>
where
    T: 'static,
{
    pub fn new(id: &str, inverse_fn: Option<fn(T)->()>) -> AtomStateAccess<T,U,A> {
        AtomStateAccess {
            id: id.to_string(),
            inverse_fn,
            _phantom_data_stored_type: PhantomData,
            _phantom_data_undo: PhantomData,
            _phantom_data_accessor_type: PhantomData,
        }
    }

    // stores a value of type T in a backing Store
    pub fn set(self, value: T) where Self :OverloadedUpdateStateAccess<T>{
        self.overloaded_set(value);
    }


    pub fn remove(self) -> Option<T> {
        remove_atom_state_with_id(&self.id)
    }

    pub fn delete(self) {
        self.remove();
    }

    pub fn setit() {
        
    }


    pub fn undo(&self) where Self: OverloadedUpdateStateAccess<T> {
        self.overloaded_undo();
    }

    /// updates the stored state in place
    /// using the provided function
    pub fn update<F: FnOnce(&mut T) -> ()>(&self, func: F) where Self :OverloadedUpdateStateAccess<T>{
        
        self.overloaded_update( func);
    }

    pub fn state_exists(self) -> bool {
        atom_state_exists_for_id::<T>(&self.id)
    }

    pub fn get_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        read_atom_state_with_id(&self.id, func)
    }
}


// If the stored type is clone, then implement clone for AtomStateAccess
pub trait CloneAtomState<T>
where
    T: Clone + 'static,
{
    fn get(&self) -> T;
    fn soft_get(&self) -> Option<T>;
}

impl<T,U,A> CloneAtomState<T> for AtomStateAccess<T,U,A>
where
    T: Clone + 'static,
{
    /// returns a clone of the stored state panics if not stored.
    fn get(&self) -> T {
        clone_atom_state_with_id::<T>(&self.id).expect("state should be present")
    }

    fn soft_get(&self) -> Option<T> {
        clone_atom_state_with_id::<T>(&self.id)
    }
}

// If the accessor type is Atom, and Undo type is Allow Undo, then 
// ensure that updates cause an undo to be appended.
pub trait  OverloadedUpdateStateAccess<T> where T:'static {
    fn overloaded_update<F: FnOnce(&mut T) -> ()>(&self, func: F);
       
    fn overloaded_undo(&self);
    fn overloaded_set(self, value: T);      
}


impl <T> OverloadedUpdateStateAccess<T> for AtomStateAccess<T,NoUndo,IsAnAtomState> where T:'static
{
    fn overloaded_undo(&self){
        panic!("cannot undo this atom is not undoable");
    }
        
    fn overloaded_update<F: FnOnce(&mut T) -> ()>(&self, func: F) {
        update_atom_state_with_id(&self.id, func);
    }

    fn overloaded_set(self, value: T) {
        set_atom_state_with_id(value, &self.id);
    }
}


impl <T> OverloadedUpdateStateAccess<T> for AtomStateAccess<T,AllowUndo,IsAnAtomState>
where T:Clone + 'static,
{
    fn overloaded_undo(&self){
        
        undo_atom_state::<T,AllowUndo,IsAnAtomState>(&self.id)
    }

    fn overloaded_update<F: FnOnce(&mut T) -> ()>(&self, func: F) {
        update_atom_state_with_id_with_undo(&self.id, func);
    }

    fn overloaded_set(self, value: T) {
        set_atom_state_with_id_with_undo(value, &self.id);
    }
}



impl <T> OverloadedUpdateStateAccess<T> for AtomStateAccess<T,NoUndo,IsAComputedState>
where T:Clone + 'static,
{
    fn overloaded_undo(&self){
        panic!("You cannot undo a computed state!")
    }

    fn overloaded_update<F: FnOnce(&mut T) -> ()>(&self, _func: F) {
       panic!("You cannot update a computed function")
    }

    fn overloaded_set(self, value: T) {
        if let Some(inverse_fn) = self.inverse_fn {
            inverse_fn(value)
        } else {
            panic!("You cannot set computed state with an inverse function being set ")
        }
    }
}
// If the underlying stored type is Clone and PartialEq
// `changed()` will return true the first time called and then false
// if called again with the same content.
#[derive(Clone)]
struct ChangedWrapper<T>(T);

pub trait ChangedAtomState {
    fn changed(&self) -> bool;
}

impl<T,U,A> ChangedAtomState for AtomStateAccess<T,U,A>
where
    T: Clone + 'static + PartialEq,
{
    fn changed(&self) -> bool {
        if atom_state_exists_for_id::<ChangedWrapper<T>>(&self.id){
            read_atom_state_with_id::<ChangedWrapper<T>,_,_>(&self.id, |old|
                self.get_with(|current| &old.0==current )
            )
        } else {
            set_atom_state_with_id(ChangedWrapper(self.get()), &self.id);
            true
        }
    }
}
// If the underlying type provides display then so does the AtomStateAccess
impl<T,U,A> std::fmt::Display for AtomStateAccess<T,U,A>
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

impl<T,U,A> Add for AtomStateAccess<T,U,A>
where
    T: Copy + Add<Output = T> + 'static,
{
    type Output = T;

    fn add(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o + *s))
    }
}

impl<T,U,A> Mul for AtomStateAccess<T,U,A>
where
    T: Copy + Mul<Output = T> + 'static,
{
    type Output = T;

    fn mul(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o * *s))
    }
}

impl<T,U,A> Div for AtomStateAccess<T,U,A>
where
    T: Copy + Div<Output = T> + 'static,
{
    type Output = T;

    fn div(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o / *s))
    }
}

impl<T,U,A> Sub for AtomStateAccess<T,U,A>
where
    T: Copy + Sub<Output = T> + 'static,
{
    type Output = T;

    fn sub(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o - *s))
    }
}
