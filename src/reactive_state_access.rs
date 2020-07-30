use crate::store::RxFunc;
use crate::reactive_state_functions::*;
use std::marker::PhantomData;
use crate::observable::Observable;
use crate::store::StorageKey;
// use seed::prelude::*;
// marker types



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
            id: self.id,
            
            
            _phantom_data_stored_type: PhantomData::<T>,

        }
    }
}


impl<T> Copy for Atom<T> {}



impl<T> Atom<T>
where
    T: 'static,
{
    pub fn new(id: StorageKey) -> Atom<T> {
        Atom {
            id,
            _phantom_data_stored_type: PhantomData,
        }
    }

    // stores a value of type T in a backing Store
    pub fn inert_set(self, value: T) where T:'static{
        set_inert_atom_state_with_id(value, self.id);
    }
    // stores a value of type T in a backing Store
    pub fn set(self, value: T) where T:'static{
        set_atom_state_with_id(value, self.id);
    }

    pub fn update<F: FnOnce(&mut T) -> ()>(&self, func: F) where T:'static{
        update_atom_state_with_id(self.id, func);
    }
    pub fn id(&self) -> StorageKey{self.id}

    pub fn remove(self) -> Option<T> {
        remove_reactive_state_with_id(self.id)
    }

    pub fn delete(self) {
        self.remove();
    }
   
    pub fn reset_to_default(&self) {
        (clone_reactive_state_with_id::<RxFunc>(self.id).unwrap().func)();
        execute_reaction_nodes(&self.id);
    }

    pub fn state_exists(self) -> bool {
        reactive_state_exists_for_id::<T>(self.id)
    }

    pub fn get_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        read_reactive_state_with_id(self.id, func)
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



pub struct AtomUndo<T> where T:Clone {
    pub id: StorageKey,
    pub _phantom_data_stored_type: PhantomData<T>,
}


impl<T> std::fmt::Debug for AtomUndo<T> where T:Clone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:#?})", self.id)
    }
}


impl<T> Clone for AtomUndo<T> where T:Clone {
    fn clone(&self) -> AtomUndo<T> {
        AtomUndo::<T> {
            id: self.id,
            
            
            _phantom_data_stored_type: PhantomData::<T>,

        }
    }
}


impl<T> Copy for AtomUndo<T> where T:Clone {}



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
    pub fn inert_set(self, value: T) where T:'static{
        set_inert_atom_state_with_id_with_undo(value, self.id);
    }
    // stores a value of type T in a backing Store
    pub fn set(self, value: T) where T:'static{
        set_atom_state_with_id_with_undo(value, self.id);
    }

    pub fn update<F: FnOnce(&mut T) -> ()>(&self, func: F) where T:'static{
        update_atom_state_with_id_with_undo(self.id, func);
    }
    pub fn id(&self) -> StorageKey{self.id}

    pub fn remove(self) -> Option<T> {
        remove_reactive_state_with_id_with_undo(self.id)
    }

    pub fn delete(self) {
        self.remove();
    }
   
    pub fn reset_to_default(&self) {
        (clone_reactive_state_with_id::<RxFunc>(self.id).unwrap().func)();
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

    pub fn force_trigger(&self){
        (clone_reactive_state_with_id::<RxFunc>(self.id).unwrap().func)();

    }
   

    pub fn state_exists(self) -> bool {
        reactive_state_exists_for_id::<T>(self.id)
    }

    pub fn get_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        read_reactive_state_with_id(self.id, func)
    }
    
    #[topo::nested]
    pub fn on_update<F: FnOnce() -> R,R>(&self, func:F) -> Option<R> {
        let first_call_accessor = crate::hooks_state_functions::use_state(||true);
        let mut recalc = false ;

        self.observe_with(|_| 
            if first_call_accessor.get() {
                first_call_accessor.set(false)
            } else {
                recalc = true
            }
        );
        if recalc {
            Some(func())
        } else {
            None
        }
    }
    
    #[topo::nested]
    pub fn has_updated(&self) -> bool {
        let first_call_accessor = crate::hooks_state_functions::use_state(||true);
        let mut recalc = false ;
        
        self.observe_with(|_| 
            if first_call_accessor.get() {
                first_call_accessor.set(false)
            } else {
                recalc = true
            }
        );
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
T: Clone + 'static + PartialEq,{
    fn observe_change(&self)  -> (Option<T>,T);
    fn has_changed(&self)  -> bool;
    fn on_change<F: FnOnce(&T,&T)-> R, R>(&self, func: F)  -> R;
}

use crate::state_access::CloneState;

// The below is broke as need None if no prior state
impl<T> ObserveChangeReactiveState<T> for Atom<T>
where
T: Clone + 'static + PartialEq,{
    #[topo::nested]
    fn observe_change(&self)  -> (Option<T>,T){
        let previous_value_access = crate::hooks_state_functions::use_state(|| self.get() );
        previous_value_access.get_with(|previous_value|
         self.observe_with(|new_value|
            if *previous_value != *new_value {
                previous_value_access.set(new_value.clone());
                (Some(previous_value.clone()),new_value.clone())
            } else {
                (None,new_value.clone())
            }
         )
    )
    }

    #[topo::nested]
    fn has_changed(&self)  -> bool{
        let previous_value_access = crate::hooks_state_functions::use_state(|| self.get() );
        previous_value_access.get_with(|previous_value|
            self.observe_with(|new_value| new_value!= previous_value)
        )
    }
            
    fn on_change<F: FnOnce(&T,&T)-> R, R>(&self, func: F)  -> R {
            let previous_value_access = crate::hooks_state_functions::use_state(|| self.get() );
            previous_value_access.get_with(|previous_value|
                self.observe_with(|new_value|
                    func(previous_value,new_value)

                )
            )
        }
}



impl<T> ObserveChangeReactiveState<T> for Reaction<T>
where
T: Clone + 'static + PartialEq,{
    #[topo::nested]
    fn observe_change(&self)  -> (Option<T>,T){
        let previous_value_access = crate::hooks_state_functions::use_state(|| self.get() );
        previous_value_access.get_with(|previous_value|
         self.observe_with(|new_value|
            if *previous_value != *new_value {
                previous_value_access.set(new_value.clone());
                (Some(previous_value.clone()),new_value.clone())
            } else {
                (None,new_value.clone())
            }
         )
    )
    }

    #[topo::nested]
    fn has_changed(&self)  -> bool{
        let previous_value_access = crate::hooks_state_functions::use_state(|| self.get() );
        previous_value_access.get_with(|previous_value|
            self.observe_with(|new_value| new_value!= previous_value)
        )
    }
            
    fn on_change<F: FnOnce(&T,&T)-> R, R>(&self, func: F)  -> R {
            let previous_value_access = crate::hooks_state_functions::use_state(|| self.get() );
            previous_value_access.get_with(|previous_value|
                self.observe_with(|new_value|
                    func(previous_value,new_value)

                )
            )
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
