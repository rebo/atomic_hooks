
use crate::reactive_state_access::{ReactiveStateAccess};
use crate::state_access::StateAccess;
use crate::marker::*;
use std::cell::RefCell;

use crate::store::{ReactiveContext};
use crate::reactive_state_functions::{STORE,clone_reactive_state_with_id,read_reactive_state_with_id};

pub trait Observable<T> where T:'static {
    // fn id(&self) -> StorageKey;
    fn observe(&self) -> T where T:Clone + 'static;
    fn observe_with<F: FnOnce(&T)-> R,R >(&self, func:F) -> R ;
}

impl <T,U,A>Observable<T> for  ReactiveStateAccess<T,U,A> where T:'static {
    fn observe(&self) -> T where T:Clone {
        
    let context = illicit::Env::get::<RefCell<ReactiveContext>>()
    .expect("No #[reaction] context found, are you sure you are in one? I.e. does the current function have a #[reaction] tag?");
    context.borrow_mut().reactive_state_accessors.push(self.id);

    STORE.with(|store_refcell| {
    store_refcell
        .borrow_mut()
        .add_dependency(&self.id, &context.borrow().key);
    });

    clone_reactive_state_with_id::<T>(&self.id).unwrap()
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
        read_reactive_state_with_id(&self.id, func)
    
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

    clone_reactive_state_with_id::<T>(&id).unwrap()
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
        read_reactive_state_with_id(&id, func)
    
}

}


pub struct AtomVec<T> where T:'static {
    vec: ReactiveStateAccess::<Vec<T>, NoUndo,IsAnAtomState>,
    filtered_idxs: Vec<usize>,
}

impl  <T>AtomVec<T> where T:'static {
    pub  fn iter(&self) -> AtomVecIterator<T> {
        AtomVecIterator::new(self.clone())
    }

    pub fn sort(&mut self) where T:PartialEq +Eq + PartialOrd{
        self.vec.clone().get_with(|vec|            
            self.filtered_idxs.sort_by(|&a,&b|
                (vec[a].partial_cmp(&vec[b])).unwrap()
            )
        );  
    }

    
    pub fn filter<F:FnMut(&T)-> bool>(&mut self , mut func: F ) {
        self.vec.clone().get_with(|vec|
            self.filtered_idxs = self.filtered_idxs
                .iter()
                .filter(|&&idx| 
                    func(vec.get(idx).unwrap())
                )
                .cloned().collect()
        );  
    }

    // pub fn split<F>(&self, mut pred: F) -> (AtomVec<T>,AtomVec<T>) where F: FnMut(&T) -> bool, {
    //     self.vec.get_with(|vec|
    //         {let split_vecs = self.filtered_idxs
    //             .split(
    //                 |&idx| 
    //                     pred(vec.get(idx).unwrap())
    //             )
    //             .collect::<Vec<_>>();
    //             let atom_vec_0 = AtomVec {
    //                 vec : self.vec,
    //                 filtered_idxs: split_vecs[0].to_vec()
    //             };
    //             let atom_vec_1 = AtomVec {
    //                 vec : self.vec,
    //                 filtered_idxs: split_vecs[1].to_vec()
    //             };
    //             (atom_vec_0,atom_vec_1)
    //         }
    //     )
    // }

}

impl <T>Clone for AtomVec<T> where T:'static { 
    fn clone(&self) -> Self {
        AtomVec {
            vec: self.vec,
            filtered_idxs: self.filtered_idxs.clone()
        }
    }
}

pub struct AtomVecIterator<T> where T:'static{
    atom_vec : AtomVec<T>,
    current: usize 
}

impl <T>AtomVecIterator<T> {
    pub fn new(atom_vec: AtomVec<T>) -> AtomVecIterator::<T>{
    AtomVecIterator{
        atom_vec,
        current: 0
    }
}
}

pub trait ObservableVec<T,U,A> where T: 'static{
    fn observe_and_filter<F:FnMut( usize,&T) -> Option<usize> >(&self, func: F) -> AtomVec<T>;
}

impl <T,U,A>ObservableVec<T,U,A> for ReactiveStateAccess<Vec<T>,U,A> where T:  'static {
    fn observe_and_filter<F:FnMut( usize,&T) -> Option<usize> >(&self, func: F) -> AtomVec<T>{
        let mut func = func;
        self.observe_with( |v| {
            let filtered = v.iter()
                .enumerate()
                .filter_map(|(idx,v)| func(idx,v))
                .collect::<Vec<usize>>();
                
                AtomVec {
                    vec: ReactiveStateAccess::new(self.id),
                    filtered_idxs: filtered
                }
        })
}
}



impl <T,U,A>ObservableVec<T,U,A> for ReactiveStateAccess<AtomVec<T>,U,A> where T:  'static {
    fn observe_and_filter<F:FnMut( usize,&T) -> Option<usize> >(&self, func: F) -> AtomVec<T>{
        let mut func = func;
        let atom_vec = self.observe();

        let orig_vec = atom_vec.vec;
        let idxs = atom_vec.filtered_idxs;
        

        // we need to collate the vec items along with their original indices

        let new_idxes = orig_vec.get_with(|v|
             idxs.iter().map(|i| (i, v.get(*i).unwrap()) ).filter_map(|(idx, value)|  func(*idx, value) ).collect()
        );

        AtomVec {
            vec: orig_vec.clone(),
            filtered_idxs: new_idxes
        }

}
}


impl <T> Iterator for AtomVecIterator<T> where T:Clone {
    // we will be counting with usize
    type Item = T;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        

       let val =  if let Some(idx) = self.atom_vec.filtered_idxs.get(self.current) {
            self.atom_vec.vec.get_with(|v| v.get(*idx).cloned())
        } else {
            None
        };
        // Increment our count. This is why we started at zero.
        self.current += 1;
        val
    }
}


// impl <T,U,A>Observable<T> for  ReactiveStateAccess<T,U,A> {
//     fn id(&self) -> StorageKey{
//         self.id
//     }
// }

// impl <T>Observable<T> for  StateAccess<T> {
//     fn id(&self) -> StorageKey{
//         StorageKey::TopoKey(self.id)
//     }
// }