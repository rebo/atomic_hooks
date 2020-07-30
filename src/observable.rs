
use crate::reactive_state_access::{CloneReactiveState, Atom,AtomUndo,Reaction};
use crate::state_access::{StateAccess,CloneState};

use std::cell::RefCell;

use crate::store::{ReactiveContext};
use crate::reactive_state_functions::{STORE,clone_reactive_state_with_id,read_reactive_state_with_id};

pub trait Observable<T> where T:'static {
    // fn id(&self) -> StorageKey;
    fn observe(&self) -> T where T:Clone + 'static;
    fn observe_update(&self) -> (Option<T>,T) where T:Clone + 'static;
    fn observe_with<F: FnOnce(&T)-> R,R >(&self, func:F) -> R ;
}

impl <T>Observable<T> for  Atom<T> where T:'static  {
    fn observe(&self) -> T where T:'static + Clone {
        
    let context = illicit::Env::get::<RefCell<ReactiveContext>>()
    .expect("No #[reaction] context found, are you sure you are in one? I.e. does the current function have a #[reaction] tag?");
    context.borrow_mut().reactive_state_accessors.push(self.id);

    STORE.with(|store_refcell| {
    store_refcell
        .borrow_mut()
        .add_dependency(&self.id, &context.borrow().key);
    });

    clone_reactive_state_with_id::<T>(self.id).unwrap()
    }

    #[topo::nested]
    fn observe_update(&self)  -> (Option<T>,T)where T:'static + Clone {
        let previous_value_access = crate::hooks_state_functions::use_state(|| None );
        let opt_previous_value = previous_value_access.get();
        let new_value = self.get();
        previous_value_access.set(Some(new_value.clone()));
        (opt_previous_value,new_value)
    }

 fn observe_with<F: FnOnce(&T)-> R,R >(&self, func:F) -> R {
    if let Some(context) =   illicit::Env::get::<RefCell<ReactiveContext>>() {
        context.borrow_mut().reactive_state_accessors.push(self.id.clone());

        STORE.with(|store_refcell| {
            store_refcell
                .borrow_mut()
                .add_dependency(&self.id, &context.borrow().key);
        });
    }
        read_reactive_state_with_id(self.id, func)
    
}

}


impl <T>Observable<T> for  AtomUndo<T> where T:'static +Clone {
    fn observe(&self) -> T where T:'static + Clone {
        
    let context = illicit::Env::get::<RefCell<ReactiveContext>>()
    .expect("No #[reaction] context found, are you sure you are in one? I.e. does the current function have a #[reaction] tag?");
    context.borrow_mut().reactive_state_accessors.push(self.id);

    STORE.with(|store_refcell| {
    store_refcell
        .borrow_mut()
        .add_dependency(&self.id, &context.borrow().key);
    });

    clone_reactive_state_with_id::<T>(self.id).unwrap()
    }

    #[topo::nested]
    fn observe_update(&self)  -> (Option<T>,T)where T:'static + Clone {
        let previous_value_access = crate::hooks_state_functions::use_state(|| None );
        let opt_previous_value = previous_value_access.get();
        let new_value = self.get();
        previous_value_access.set(Some(new_value.clone()));
        (opt_previous_value,new_value)
    }



// <T: 'static, F: FnOnce(&T) -> R, R>(id: StorageKey, func: F) -> R {
 fn observe_with<F: FnOnce(&T)-> R,R >(&self, func:F) -> R {
    if let Some(context) =   illicit::Env::get::<RefCell<ReactiveContext>>() {
        context.borrow_mut().reactive_state_accessors.push(self.id.clone());

        STORE.with(|store_refcell| {
            store_refcell
                .borrow_mut()
                .add_dependency(&self.id, &context.borrow().key);
        });
    }
        read_reactive_state_with_id(self.id, func)
    
}

}



impl <T>Observable<T> for  Reaction<T> where T:'static {
    fn observe(&self) -> T where T:Clone {
        
    let context = illicit::Env::get::<RefCell<ReactiveContext>>()
    .expect("No #[reaction] context found, are you sure you are in one? I.e. does the current function have a #[reaction] tag?");
    context.borrow_mut().reactive_state_accessors.push(self.id);

    STORE.with(|store_refcell| {
    store_refcell
        .borrow_mut()
        .add_dependency(&self.id, &context.borrow().key);
    });

    clone_reactive_state_with_id::<T>(self.id).unwrap()
    }

    #[topo::nested]
    fn observe_update(&self)  -> (Option<T>,T)where T:'static + Clone {
        let previous_value_access = crate::hooks_state_functions::use_state(|| None );
        let opt_previous_value = previous_value_access.get();
        let new_value = self.get();
        previous_value_access.set(Some(new_value.clone()));
        (opt_previous_value,new_value)
    }


// <T: 'static, F: FnOnce(&T) -> R, R>(id: StorageKey, func: F) -> R {
 fn observe_with<F: FnOnce(&T)-> R,R >(&self, func:F) -> R {
    if let Some(context) =   illicit::Env::get::<RefCell<ReactiveContext>>() {
        context.borrow_mut().reactive_state_accessors.push(self.id.clone());

        STORE.with(|store_refcell| {
            store_refcell
                .borrow_mut()
                .add_dependency(&self.id, &context.borrow().key);
        });
    }
        read_reactive_state_with_id(self.id, func)
    
}

}


impl <T>Observable<T> for  StateAccess<T> where T:'static {
    fn observe(&self) -> T where T:Clone {
        let id = crate::store::StorageKey::TopoKey(self.id);
    let context = illicit::Env::get::<RefCell<ReactiveContext>>()
    .expect("No #[reaction] context found, are you sure you are in one? I.e. does the current function have a #[reaction] tag?");
    context.borrow_mut().reactive_state_accessors.push(id);

    STORE.with(|store_refcell| {
    store_refcell
        .borrow_mut()
        .add_dependency(&id, &context.borrow().key);
    });

    clone_reactive_state_with_id::<T>(id).unwrap()
    }

    #[topo::nested]
    fn observe_update(&self)  -> (Option<T>,T)where T:'static + Clone {
        let previous_value_access = crate::hooks_state_functions::use_state(|| None );
        let opt_previous_value = previous_value_access.get();
        let new_value = self.get();
        previous_value_access.set(Some(new_value.clone()));
        (opt_previous_value,new_value)
    }


// <T: 'static, F: FnOnce(&T) -> R, R>(id: StorageKey, func: F) -> R {
 fn observe_with<F: FnOnce(&T)-> R,R >(&self, func:F) -> R {
    let id = crate::store::StorageKey::TopoKey(self.id);
    if let Some(context) =   illicit::Env::get::<RefCell<ReactiveContext>>() {
        context.borrow_mut().reactive_state_accessors.push(id.clone());

        STORE.with(|store_refcell| {
            store_refcell
                .borrow_mut()
                .add_dependency(&id, &context.borrow().key);
        });
    }
        read_reactive_state_with_id(id, func)
    
}

}
