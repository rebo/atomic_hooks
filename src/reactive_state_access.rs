use crate::store::Reaction;
use crate::reactive_state_functions::*;
use std::marker::PhantomData;
use crate::observable::Observable;
use crate::store::StorageKey;
use crate::marker::*;
// use seed::prelude::*;
// marker types


///  Accessor struct that provides access to getting and setting the
///  state of the stored type
///
// #[derive(Copy)]
pub struct ReactiveStateAccess<T,U,A> {
    pub id: StorageKey,

    pub _phantom_data_stored_type: PhantomData<T>,
    pub _phantom_data_undo : PhantomData<U>,
    pub _phantom_data_accessor_type : PhantomData<A>,
}

impl<T,U,A> std::fmt::Debug for ReactiveStateAccess<T,U,A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:#?})", self.id)
    }
}
// 
impl<T,U,A> Clone for ReactiveStateAccess<T,U,A> {
    fn clone(&self) -> ReactiveStateAccess<T,U,A> {
        ReactiveStateAccess::<T,U,A> {
            id: self.id,
            
            _phantom_data_stored_type: PhantomData::<T>,
            _phantom_data_undo : PhantomData::<U>,
            _phantom_data_accessor_type : PhantomData::<A>,
        }
    }
}


impl<T,U,A> Copy for ReactiveStateAccess<T,U,A> {}

impl<T,U,A> ReactiveStateAccess<T,U,A>
where
    T: 'static,
{
    pub fn new(id: StorageKey) -> ReactiveStateAccess<T,U,A> {
        ReactiveStateAccess {
            id,
            
            _phantom_data_stored_type: PhantomData,
            _phantom_data_undo: PhantomData,
            _phantom_data_accessor_type: PhantomData,
        }
    }

    // stores a value of type T in a backing Store
    pub fn inert_set(self, value: T) where Self :OverloadedUpdateStateAccess<T>{
        self.overloaded_inert_set(value);
    }


    // stores a value of type T in a backing Store
    pub fn set(self, value: T) where Self :OverloadedUpdateStateAccess<T>{
        self.overloaded_set(value);
    }



    pub fn remove(self) -> Option<T> {
        remove_reactive_state_with_id(&self.id)
    }

    pub fn delete(self) {
        self.remove();
    }


    pub fn undo(&self) where Self: OverloadedUpdateStateAccess<T> {
        self.overloaded_undo();
    }

    /// updates the stored state in place
    /// using the provided function
    pub fn update<F: FnOnce(&mut T) -> ()>(&self, func: F) where Self :OverloadedUpdateStateAccess<T>{
        self.overloaded_update( func); 
    }

    pub fn reset_to_default(&self) where Self :OverloadedUpdateStateAccess<T>{
        self.overloaded_reset_to_default(); 
    }


    pub fn state_exists(self) -> bool {
        reactive_state_exists_for_id::<T>(&self.id)
    }

    pub fn get_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        read_reactive_state_with_id(&self.id, func)
    }



    pub fn on_update<F: FnOnce() -> R,R>(&self, func:F) -> Option<R> {
        let mut recalc = false ;
        self.observe_with(|_| recalc = true);
        if recalc {
            Some(func())
        } else {
            None
        }
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

impl<T,U,A> CloneReactiveState<T> for ReactiveStateAccess<T,U,A>
where
    T: Clone + 'static,
{
    /// returns a clone of the stored state panics if not stored.
    fn get(&self) -> T {
        clone_reactive_state_with_id::<T>(&self.id).expect("state should be present")
    }

    fn soft_get(&self) -> Option<T> {
        clone_reactive_state_with_id::<T>(&self.id)
    }
}

// If the accessor type is Atom, and Undo type is Allow Undo, then 
// ensure that updates cause an undo to be appended.
pub trait  OverloadedUpdateStateAccess<T> where T:'static {
    fn overloaded_update<F: FnOnce(&mut T) -> ()>(&self, func: F);
    fn overloaded_reset_to_default(&self);   
    fn overloaded_undo(&self);
    fn overloaded_inert_set(self, value: T);      
    fn overloaded_set(self, value: T);
}


impl <T> OverloadedUpdateStateAccess<T> for ReactiveStateAccess<T,NoUndo,IsAnAtomState> where T:'static
{
    fn overloaded_undo(&self){
        panic!("cannot undo this atom is not undoable");
    }


    fn overloaded_reset_to_default(&self){
        // execute_reaction_nodes(&self.id);
        (clone_reactive_state_with_id::<Reaction>(&self.id).unwrap().func)();
        execute_reaction_nodes(&self.id);
    
    }

    
        
    fn overloaded_update<F: FnOnce(&mut T) -> ()>(&self, func: F) {

        update_atom_state_with_id(&self.id, func);

    }

    fn overloaded_inert_set(self, value: T) {
        set_inert_atom_state_with_id(value, &self.id);
    }

    fn overloaded_set(self, value: T) {
        set_atom_state_with_id(value, &self.id);
    }
}


impl <T> OverloadedUpdateStateAccess<T> for ReactiveStateAccess<T,AllowUndo,IsAnAtomState>
where T:Clone + 'static,
{

    fn overloaded_reset_to_default(&self){
        (clone_reactive_state_with_id::<Reaction>(&self.id).unwrap().func)();
        execute_reaction_nodes(&self.id);
    }

    fn overloaded_undo(&self){
        
        undo_atom_state::<T,AllowUndo,IsAnAtomState>(&self.id)
    }

    fn overloaded_update<F: FnOnce(&mut T) -> ()>(&self, func: F) {
        update_atom_state_with_id_with_undo(&self.id, func);
    }

    fn overloaded_inert_set(self, value: T) {
        set_inert_atom_state_with_id_with_undo(value, &self.id);
    }

    fn overloaded_set(self, value: T) {
        set_atom_state_with_id_with_undo(value, &self.id);
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

impl<T,U,A> ChangedAtomState for ReactiveStateAccess<T,U,A>
where
    T: Clone + 'static + PartialEq,
{
    fn changed(&self) -> bool {
        if reactive_state_exists_for_id::<ChangedWrapper<T>>(&self.id){
            read_reactive_state_with_id::<ChangedWrapper<T>,_,_>(&self.id, |old|
                self.get_with(|current| &old.0==current )
            )
        } else {
            set_inert_atom_state_with_id(ChangedWrapper(self.get()), &self.id);
            true
        }
    }
}
// If the underlying type provides display then so does the ReactiveStateAccess
impl<T,U,A> std::fmt::Display for ReactiveStateAccess<T,U,A>
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

impl<T,U,A> Add for ReactiveStateAccess<T,U,A>
where
    T: Copy + Add<Output = T> + 'static,
{
    type Output = T;

    fn add(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o + *s))
    }
}

impl<T,U,A> Mul for ReactiveStateAccess<T,U,A>
where
    T: Copy + Mul<Output = T> + 'static,
{
    type Output = T;

    fn mul(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o * *s))
    }
}

impl<T,U,A> Div for ReactiveStateAccess<T,U,A>
where
    T: Copy + Div<Output = T> + 'static,
{
    type Output = T;

    fn div(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o / *s))
    }
}

impl<T,U,A> Sub for ReactiveStateAccess<T,U,A>
where
    T: Copy + Sub<Output = T> + 'static,
{
    type Output = T;

    fn sub(self, other: Self) -> Self::Output {
        self.get_with(|s| other.get_with(|o| *o - *s))
    }
}
