use crate::reactive_state_access:: ReactiveStateAccess;

use crate::store::{ Reaction, ReactiveContext,Store, StorageKey, SlottedKey};
use std::cell::RefCell;
use std::rc::Rc;
use std::hash::Hash;
use crate::marker::*;
// use seed::{*,prelude};


thread_local! {
    pub static STORE: RefCell<Store> = RefCell::new(Store::new());
}

// 
//  Constructs a T atom state accessor. T is stored keyed to the provided String id.
//  The accessor always references this id therefore can you can set/update/ or get this T
//  from anywhere.
// 
//   The passed closure is only used for the first initialisation of state.
//   Subsequent evaluations of this function just returns the accessor.
//   Only one type per context can be stored in this way.
// 
//
// Typically this is created via the #[atom] attribute macro
//
pub fn atom<T: 'static , F: Fn() -> () + 'static, U,A>(current_id: StorageKey, data_fn: F)  -> ReactiveStateAccess<T,U,IsAnAtomState> {
    
    // we do not need to re-initalize the atom if it already has been stored.
    if !reactive_state_exists_for_id::<T>(&current_id) {

       
        let reaction = Reaction{
            func: Rc::new(data_fn),
        };
 
        STORE.with(|store_refcell| {   
            store_refcell
                .borrow_mut()
                .new_reaction( &current_id, reaction.clone());
        });

        (reaction.func.clone())();

        STORE.with(|store_refcell| {
            store_refcell
                .borrow_mut().add_atom(&current_id);
        })

    }
    ReactiveStateAccess::new(current_id)
}

// 
//  Constructs a T reaction state accessor. T is stored keyed to the provided String id.
//  The accessor always references this id. Typically reaction values are auto
//  created based on changes to their dependencies which could be other reaction values or an
//  atom state.
//
//   The passed closure is run whenever a dependency of the reaction state has been updated.
// 
//
// Typically this is created via the #[reaction] attribute macro
//
pub fn reaction<T:'static,U,A,F: Fn()->() + 'static>(
    current_id: StorageKey, 
    data_fn: F,
    ) -> ReactiveStateAccess<T,NoUndo,IsAReactionState> {

    if !reactive_state_exists_for_id::<T>(&current_id) {
        STORE.with(|store_refcell| {

            let key = store_refcell
                .borrow_mut()
                .primary_slotmap.insert(current_id);

            store_refcell.borrow_mut().id_to_key_map.insert(current_id, key);
        });
        
    
        let reaction = Reaction{
            func: Rc::new(data_fn),
        };
 
        STORE.with(|store_refcell| {   
            store_refcell
                .borrow_mut()
                .new_reaction( &current_id, reaction.clone());
        });

        (reaction.func.clone())();

    }


    ReactiveStateAccess::<T,NoUndo,IsAReactionState>::new(current_id)
}

pub fn computed<T:'static,U,A,F: Fn()->() + 'static>(
    current_id: StorageKey, 
    data_fn: F,
    ) -> ReactiveStateAccess<T,NoUndo,IsAReactionState> {

    if !reactive_state_exists_for_id::<T>(&current_id) {
        STORE.with(|store_refcell| {

            let key = store_refcell
                .borrow_mut()
                .primary_slotmap.insert(current_id);

            store_refcell.borrow_mut().id_to_key_map.insert(current_id, key);
        });
        
    
        let reaction = Reaction{
            func: Rc::new(data_fn),
        };
 
        STORE.with(|store_refcell| {   
            store_refcell
                .borrow_mut()
                .new_reaction( &current_id, reaction.clone());
        });

        (reaction.func.clone())();

    }


    ReactiveStateAccess::<T,NoUndo,IsAReactionState>::new(current_id)
}




pub fn undo_atom_state<T: 'static + Clone, AllowUndo,IsAnAtomState>(current_id: &StorageKey){
    
    let mut undo_vec = remove_reactive_state_with_id::<UndoVec<T>>(current_id).expect("initial undo vec should be present but its not");
    
    if undo_vec.0.len() > 1 {
        let item =  undo_vec.0.pop().expect("type to exist");    
        update_atom_state_with_id(current_id,|t| *t = item);
        
    }
    set_inert_atom_state_with_id(undo_vec, current_id) ;

}

pub fn atom_with_undo<T: 'static , F: Fn() -> () + 'static, U,A>(current_id: StorageKey, data_fn: F)  -> ReactiveStateAccess<T,AllowUndo,IsAnAtomState> where T:Clone + 'static{
    
    if !reactive_state_exists_for_id::<T>(&current_id) {

        let reaction = Reaction{
            func: Rc::new(data_fn),
        };
 
        STORE.with(|store_refcell| {   
            store_refcell
                .borrow_mut()
                .new_reaction( &current_id, reaction.clone());
        });

        (reaction.func.clone())();
        
        STORE.with(|store_refcell| {
            store_refcell
                .borrow_mut().add_atom(&current_id);
        })
    }
    ReactiveStateAccess::new(current_id)
}




pub fn unlink_dead_links(id: &StorageKey){
    let context = illicit::Env::get::<RefCell<ReactiveContext>>().expect("No #[reaction] context found, are you sure you are in one? I.e. does the current function have a #[reaction] tag?");
    if reactive_state_exists_for_id::<ReactiveContext>(id) {
    read_reactive_state_with_id::<ReactiveContext,_,()>(id, |old_context| {
        
        let ids_to_remove = old_context.reactive_state_accessors.iter().filter(|a_id| !context.borrow().reactive_state_accessors.contains(a_id));
        for id_to_remove in ids_to_remove {
            STORE.with(|store_refcell| {
                store_refcell
                    .borrow_mut()
                    .remove_dependency(id_to_remove,id);
            })
        }
    }
    ) } else {
        set_inert_atom_state_with_id::<ReactiveContext>(context.borrow().clone(), id)
    }

}






pub fn set_inert_atom_state_with_id_with_undo<T: 'static>(data: T, current_id: &StorageKey) where T:Clone {
    let item = clone_reactive_state_with_id::<T>(current_id).expect("inital state needs to be present");
    let mut  undo_vec = remove_reactive_state_with_id::<UndoVec<T>>(current_id).expect("untitlal undo vec to be present");
    undo_vec.0.push(item);
    set_inert_atom_state_with_id(undo_vec, current_id) ;
    set_inert_atom_state_with_id(data, current_id);
    
}





pub fn set_atom_state_with_id_with_undo<T: 'static>(data: T, current_id: &StorageKey) where T:Clone {
    let item = clone_reactive_state_with_id::<T>(current_id).expect("inital state needs to be present");
    let mut  undo_vec = remove_reactive_state_with_id::<UndoVec<T>>(current_id).expect("untitlal undo vec to be present");
    undo_vec.0.push(item);
    set_inert_atom_state_with_id(undo_vec, current_id) ;
    set_inert_atom_state_with_id(data, current_id);
    execute_reaction_nodes(current_id);
    
}



/// Sets the state of type T keyed to the given TopoId
pub fn set_inert_atom_state_with_id<T: 'static>(data: T, current_id: &StorageKey) {
    STORE.with(|store_refcell| {
        store_refcell
            .borrow_mut()
            .set_state_with_id::<T>(data, current_id)
    })
}


/// Sets the state of type T keyed to the given TopoId
pub fn set_atom_state_with_id<T: 'static>(data: T, current_id: &StorageKey) {
    STORE.with(|store_refcell| {
        store_refcell
            .borrow_mut()
            .set_state_with_id::<T>(data, current_id)
    });

    execute_reaction_nodes(current_id);
}



pub fn reactive_state_exists_for_id<T: 'static>(id: &StorageKey) -> bool {
    STORE.with(|store_refcell| store_refcell.borrow().state_exists_with_id::<T>(id))
}


/// Clones the state of type T keyed to the given TopoId
pub fn clone_reactive_state_with_id<T: 'static + Clone>(id: &StorageKey) -> Option<T> {
    STORE.with(|store_refcell| {
        store_refcell
            .borrow_mut()
            .get_state_with_id::<T>(id)
            .cloned()
    })
}

pub fn remove_reactive_state_with_id<T: 'static>(id: &StorageKey) -> Option<T> {
    
    STORE.with(|store_refcell| {
        store_refcell
            .borrow_mut()
            .remove_state_with_id::<T>(id)
    })
}

#[derive(Clone)]
pub struct UndoVec<T>(pub Vec<T>);

pub fn update_atom_state_with_id_with_undo<T: 'static, F: FnOnce(&mut T) -> ()>(id: &StorageKey, func: F) where T:Clone{

    let mut item = remove_reactive_state_with_id::<T>(id)
        .expect("You are trying to update a type state that doesnt exist in this context!");

    
    let mut undo_vec = remove_reactive_state_with_id::<UndoVec<T>>(id)
        .expect("You are trying to update a type state that doesnt exist in this context!");
    undo_vec.0.push(item.clone());

    set_inert_atom_state_with_id(undo_vec, id);
    

    func(&mut item);
    set_inert_atom_state_with_id(item, id);
    
    execute_reaction_nodes(id);
}

pub fn execute_reaction_nodes(id: &StorageKey) {
    let ids_reactions = STORE.with(|refcell_store|{
        let mut borrow = refcell_store.borrow_mut();
        borrow.clone_dep_funcs_for_id(id)
    });

    for (key,reaction)in &ids_reactions {
        let cloned_reaction = reaction.clone();
        (cloned_reaction.func.clone())();
        execute_reaction_nodes(&key);
    }
}



pub fn update_atom_state_with_id<T: 'static, F: FnOnce(&mut T) -> ()>(id: &StorageKey, func: F) {
    let mut item = remove_reactive_state_with_id::<T>(id)
        .expect("You are trying to update a type state that doesnt exist in this context!");
    
    func(&mut item);
    

    set_inert_atom_state_with_id(item, id);
    
    //we need to get the associated data with this key
    execute_reaction_nodes(id);

}

pub fn read_reactive_state_with_id<T: 'static, F: FnOnce(&T) -> R, R>(id: &StorageKey, func: F) -> R {
    let item = remove_reactive_state_with_id::<T>(id)
        .expect("You are trying to read a type state that doesnt exist in this context!");
    let read = func(&item);
    set_inert_atom_state_with_id(item, id);
    read
}


pub fn try_read_reactive_state_with_id<T: 'static, F: FnOnce(&T) -> R, R>(id: &StorageKey, func: F) -> Option<R> {
    if let Some(item) = remove_reactive_state_with_id::<T>(id){
        let read = func(&item);
        set_inert_atom_state_with_id(item, id);
        Some(read)
    } else {
        None
    }
   
}

pub fn return_key_for_type_and_insert_if_required<T: 'static + Clone + Eq + Hash>(value: T) -> StorageKey {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hasher};

    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    let hash_id = hasher.finish();

    let key = StorageKey::SlottedKey(SlottedKey {
        location: hash_id,
        slot: 0
    });

    STORE.with(|refcell_store|
    {
        refcell_store.borrow_mut().return_key_for_type_and_insert_if_required(key,value.clone())
    })
}







// fn deep_collision_check_for_state_id<T: 'static>(id:StorageKey, item:T, stored_value:T) -> StorageKey {
//     if let Some() = remove_reactive_state_with_id::<Vec<(T,StorageKey)>(id) {

//     } else {
//         set_inert_atom_state_with_id(vec![], id);
//     }

// }