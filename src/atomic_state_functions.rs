use std::marker::PhantomData;
// use crate::atomic_state_access::CloneAtomicState;
use crate::atomic_state_access::AtomicStateAccess;
use crate::atomic_store::{Selector,Getter,AtomicStore};
use std::cell::RefCell;

// use slotmap::{DenseSlotMap,DefaultKey, Key, SecondaryMap, SlotMap};

thread_local! {
    static ATOMIC_STORE: RefCell<AtomicStore> = RefCell::new(AtomicStore::new());
}

// 
//  Constructs a T accessor. T is stored keyed to the current topological context.
//  The accessor always references this context therefore can you can set/update/ or get this T
//  from anywhere.
// 
//   The passed closure is only used for the first initialisation of state.
//   Subsequent evaluations of this function just returns the accessor.
//   Only one type per context can be stored in this way.
// 
//  # Examples
// 
//  ```
//  let my_string =  use_state(|| "foo".to_string());
//  ...
//  ...
//   // Maybe in a Callback...
//  my_string.set("bar")
//  ```
// 
//  This stores a string "foo" in the current topological context,
//  which is later set to "bar", in some other part of the program.
// 
//  You can store Clone or non-Clone types. Although non-Clone types need
//  to be read via their accessor in a more restrictive way.
// in a parent context.

///
// if let Some(sa) = illicit::Env::get::<StateAccess<#view_builder<T>>>() {
    // illicit::child_env!(StateAccess<#view_builder<_>> => sa_builder).enter(|| {
pub fn atom<T: 'static , F: FnOnce() -> T  >(current_id: &str, data_fn: F)  -> AtomicStateAccess<T> {
    
    if !atomic_state_exists_for_id::<T>(current_id) {
        set_atomic_state_with_id::<T>(data_fn(), current_id);
        ATOMIC_STORE.with(|store_refcell| {
            store_refcell
                .borrow_mut().add_atom(current_id);
        })
    }
    AtomicStateAccess::new(current_id)
}
pub fn get<T>(access : AtomicStateAccess<T>) -> T where T:Clone + 'static{
    let getter =   illicit::Env::get::<RefCell<Getter>>().unwrap();
    getter.borrow_mut().atomic_state_accessors.push(access.id.clone());

    ATOMIC_STORE.with(|store_refcell| {
        store_refcell
            .borrow_mut()
            .add_dependency(&access.id, &getter.borrow().selector_key)
    });

    clone_atomic_state_with_id::<T>(&access.id).unwrap()
}


pub fn selector<T: 'static, F: FnOnce()-> T+ Clone + 'static>(
    current_id: &str, 
    data_fn: F) -> AtomicStateAccess<T> {
    
    let selector_sm_key =  ATOMIC_STORE.with(|store_refcell| {
        let key = store_refcell
            .borrow_mut()
            .primary_slotmap.insert(current_id.clone().to_string());

        store_refcell.borrow_mut().id_to_key_map.insert(current_id.to_string(), key);
        key
    });
    
    
    let id = current_id.to_string();
    let selector = Selector::<T,_>{
        func: move ||{
            let getter = Getter::new(&id);
            illicit::child_env!( RefCell<Getter> => RefCell::new(getter) ).enter(|| {
            set_atomic_state_with_id::<T>(data_fn(),&id);
        })
    },
    _phantom_data:PhantomData,
    };

    ((selector.func).clone())();

    ATOMIC_STORE.with(|store_refcell| {
        store_refcell
            .borrow_mut()
            .new_selector( selector_sm_key, Box::new(selector))
    });



    AtomicStateAccess::new(current_id)
}



///
///  Uses the current topological id to create a new state accessor
///
///
pub fn use_atomic_state_current<T: 'static >(current_id: &str, data: T) -> AtomicStateAccess<T> {
    
    if !atomic_state_exists_for_id::<T>(current_id) {
        set_atomic_state_with_id::<T>(data, current_id);
    }

    AtomicStateAccess::new(current_id)
}


/// Sets the state of type T keyed to the given TopoId
pub fn set_atomic_state_with_id<T: 'static>(data: T, current_id: &str) {
    ATOMIC_STORE.with(|store_refcell| {
        store_refcell
            .borrow_mut()
            .set_state_with_id::<T>(data, current_id)
    })
}

pub fn atomic_state_exists_for_id<T: 'static>(id: &str) -> bool {
    ATOMIC_STORE.with(|store_refcell| store_refcell.borrow().state_exists_with_id::<T>(id))
}


/// Clones the state of type T keyed to the given TopoId
pub fn clone_atomic_state_with_id<T: 'static + Clone>(id: &str) -> Option<T> {
    ATOMIC_STORE.with(|store_refcell| {
        store_refcell
            .borrow_mut()
            .get_state_with_id::<T>(id)
            .cloned()
    })
}

pub fn remove_atomic_state_with_id<T: 'static>(id: &str) -> Option<T> {
    ATOMIC_STORE.with(|store_refcell| {
        store_refcell
            .borrow_mut()
            .remove_state_with_id::<T>(id)
    })
}

// Provides mutable access to the stored state type T.
//
// Example:
//
// ```
// update_state_with_topo_id::<Vec<String>>( topo::Id::current(), |v|
//     v.push("foo".to_string()
// )
//
pub fn update_atomic_state_with_id<T: 'static, F: FnOnce(&mut T) -> ()>(id: &str, func: F) {
    let mut item = remove_atomic_state_with_id::<T>(id)
        .expect("You are trying to update a type state that doesnt exist in this context!");
    func(&mut item);
    set_atomic_state_with_id(item, id);

    //we need to get the associated data with this key
    
    
        let ids_selectors = ATOMIC_STORE.with(|refcell_store|{
            let mut borrow = refcell_store.borrow_mut();
            
            borrow.clone_dep_funcs_for_id(id)
        });

        for (id,selector) in ids_selectors {
            // println!("calling selector for {}", id);
            selector.calc();
            set_atomic_state_with_id(selector, &id);
        }
    

}

pub fn read_atomic_state_with_id<T: 'static, F: FnOnce(&T) -> R, R>(id: &str, func: F) -> R {
    let item = remove_atomic_state_with_id::<T>(id)
        .expect("You are trying to read a type state that doesnt exist in this context!");
    let read = func(&item);
    set_atomic_state_with_id(item, id);
    read
}
