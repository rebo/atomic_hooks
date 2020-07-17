pub use std::collections::HashSet;
use anymap::any::Any;
use slotmap::{DenseSlotMap,DefaultKey, Key, SecondaryMap};
use std::collections::HashMap;
// use seed::*;
use std::rc::Rc;
use std::hash::Hash;


#[derive(Debug,Clone)]
pub struct ReactiveContext {
    pub key: StorageKey,
    pub reactive_state_accessors: Vec<StorageKey>,
    pub always_run : bool,
}

impl ReactiveContext {
    pub fn new(key : StorageKey) -> ReactiveContext {
        ReactiveContext{
            key,
            reactive_state_accessors:vec![],
            always_run: false,
        }
    }
}

#[derive(Clone,Copy,Eq,PartialEq,Debug, Hash)]
pub enum StorageKey
{
    SlottedKey(SlottedKey),
    TopoKey(TopoKey)
} 


#[derive(Clone,Copy,Eq,PartialEq,Debug, Hash)]
pub struct SlottedKey
{
    pub location:u64,
    pub slot:u64,
}

#[derive(Clone,Copy,Eq,PartialEq,Debug, Hash)]
pub struct TopoKey
{
    pub ctx: Option<SlottedKey>,
    pub id: topo::CallId,
}

#[derive(Clone)]
pub struct Reaction {
    pub func: Rc<dyn Fn() -> () + 'static>
}

pub struct Store {
    pub id_to_key_map: HashMap<StorageKey, DefaultKey>,
    pub primary_slotmap: DenseSlotMap<DefaultKey, StorageKey>,
    pub anymap: anymap::Map<dyn Any>,
    pub unseen_ids: HashSet<TopoKey>,
}

impl Store {
    pub(crate) fn new() -> Store {
        Store {
            id_to_key_map: HashMap::new(),
            primary_slotmap: DenseSlotMap::new(),
            anymap: anymap::Map::new(),
            unseen_ids: HashSet::new(),
        }
    }

    pub fn new_reaction(&mut self, reaction_sm_key: &StorageKey, func: Reaction){
        
        let key = self.id_to_key_map.get(reaction_sm_key).unwrap().clone();
        if let Some(map) = self.get_mut_secondarymap::<Reaction>(){
            
            map.insert(key, func);
            
        } else {
            let mut sm: SecondaryMap<DefaultKey, Reaction> = SecondaryMap::new();
            sm.insert(key, func);
            self.anymap.insert(sm);
        }
    }

    pub fn add_atom(&mut self, id: &StorageKey) {
        let dep_sm_key = self.id_to_key_map.get(id).unwrap().clone();

        if self.get_secondarymap::<Vec<DefaultKey>>().is_none(){
            self.register_secondarymap::<Vec<DefaultKey>>();
        }

        let map = &mut self.get_mut_secondarymap::<Vec<DefaultKey>>().unwrap();
        map.insert(dep_sm_key, vec![]);
    }

    pub fn remove_dependency(&mut self,source_id: &StorageKey, reaction_id:&StorageKey){

        let source_sm_key = self.id_to_key_map.get(source_id).unwrap().clone();
        let reaction_sm_key = self.id_to_key_map.get(reaction_id).unwrap().clone();
        
        
        let map = &mut self.get_mut_secondarymap::<Vec<DefaultKey>>().unwrap();
        
            if let Some(nodes) = map.get_mut(source_sm_key) {
                nodes.retain(|n| *n != reaction_sm_key);
            } else {
                panic!("Trying to remove a from a state which does not exit")
            }
    }

    pub fn add_dependency(&mut self,source_id: &StorageKey, reaction_id:&StorageKey){

        let source_sm_key = self.id_to_key_map.get(source_id).unwrap().clone();
        let reaction_sm_key = self.id_to_key_map.get(reaction_id).unwrap().clone();
        
        let map = &mut self.get_mut_secondarymap::<Vec<DefaultKey>>().unwrap();
    
        if let Some(nodes) = map.get_mut(source_sm_key) {
            if !nodes.contains(&reaction_sm_key) {
                nodes.push(reaction_sm_key)
            }
        } else {
            map.insert(source_sm_key, vec![reaction_sm_key]);
        }
    }



    pub(crate) fn state_exists_with_id<T: 'static>(&self, id: &StorageKey) -> bool {
        match (self.id_to_key_map.get(id), self.get_secondarymap::<T>()) {
            (Some(existing_key), Some(existing_secondary_map)) => {
                existing_secondary_map.contains_key(*existing_key)
            }
            (_, _) => false,
        }
    }

    pub fn get_state_with_id<T: 'static>(
        &self,
        current_id: &StorageKey,
    ) -> Option<&T> {

        match (
            self.id_to_key_map.get(current_id),
            self.get_secondarymap::<T>(),
        ) {
            (Some(existing_key), Some(existing_secondary_map)) => {
                existing_secondary_map.get(*existing_key)
            }
            (_, _) => None,
        }
    }

    pub(crate) fn remove_state_with_id<T: 'static>(
        &mut self,
        current_id: &StorageKey,
    ) -> Option<T> {
        // /self.unseen_ids.remove(&current_id);

     
        //unwrap or default to keep borrow checker happy
        let key = self
            .id_to_key_map
            .get(current_id)
            .copied()
            .unwrap_or_default();

        if key.is_null() {
            None
        } else if let Some(existing_secondary_map) = self.get_mut_secondarymap::<T>() {
            
            
            existing_secondary_map.remove(key)
        } else {
            None
        }
    }


    pub(crate) fn clone_dep_funcs_for_id(&mut self, id: &StorageKey)-> Vec<(StorageKey, Reaction )>{
        
        let  reaction_keys = self.get_state_with_id::<Vec<DefaultKey>>(id).cloned();
        
         if let Some(reaction_keys) = &reaction_keys {
        
        reaction_keys.iter().filter_map(|key|  {
            
            if let Some(existing_secondary_map) = self.get_mut_secondarymap::<Reaction>() {
                
                if let Some( reaction) =  existing_secondary_map.get(*key).cloned(){
                
               Some((self.primary_slotmap.get(*key).unwrap().clone(),reaction))
                } else {
                    panic!("cannot find {:#?} for id {:#?}",key, id);
                }
            } else {
                None
            }
        }).collect::<Vec<(StorageKey,Reaction)>>()
        }
        else {
            vec![]
        }    
    
    }

    pub(crate) fn set_state_with_id<T: 'static>(&mut self, data: T, current_id: &StorageKey) {

        
        //unwrap or default to keep borrow checker happy
        let key = self
            .id_to_key_map
            .get(current_id)
            .copied()
            .unwrap_or_default();


        if key.is_null() {
            let key = self.primary_slotmap.insert(*current_id);
            self.id_to_key_map.insert(*current_id, key);
            if let Some(sec_map) = self.get_mut_secondarymap::<T>() {
                sec_map.insert(key, data);
            } else {
                self.register_secondarymap::<T>();
                self.get_mut_secondarymap::<T>().unwrap().insert(key, data);
            }
        } else if let Some(existing_secondary_map) = self.get_mut_secondarymap::<T>() {
            existing_secondary_map.insert(key, data);
        } else {
            self.register_secondarymap::<T>();
            self.get_mut_secondarymap::<T>().unwrap().insert(key, data);
        }
    }

    pub fn get_secondarymap<T: 'static>(&self) -> Option<&SecondaryMap<DefaultKey, T>> {
        self.anymap.get::<SecondaryMap<DefaultKey, T>>()
    }

    pub fn get_mut_secondarymap<T: 'static>(&mut self) -> Option<&mut SecondaryMap<DefaultKey, T>> {
        self.anymap.get_mut::<SecondaryMap<DefaultKey, T>>()
    }

    pub fn register_secondarymap<T: 'static>(&mut self) {
        let sm: SecondaryMap<DefaultKey, T> = SecondaryMap::new();
        self.anymap.insert(sm);
    }

    pub fn return_key_for_type_and_insert_if_required<T: 'static + Clone + Eq + Hash>(&mut self, id: StorageKey, value: T) -> StorageKey {
        
        //unwrap or default to keep borrow checker happy
        let key = self
            .id_to_key_map
            .get(&id)
            .copied()
            .unwrap_or_default();


        if key.is_null() {
            let key = self.primary_slotmap.insert(id);
            self.id_to_key_map.insert(id, key);
            if let Some(sec_map) = self.get_mut_secondarymap::<T>() {

                if let Some(item)  = sec_map.get(key){
                    if item == &value {
                        id
                    } else {
                        unimplemented!() // deeper check needed here;
                    }
                } else {
                    sec_map.insert(key, value);
                    id
                }
        

            } else {
                self.register_secondarymap::<T>();
                self.get_mut_secondarymap::<T>().unwrap().insert(key,value);
                id
            }
        } else if let Some(existing_secondary_map) = self.get_mut_secondarymap::<T>() {
            
            if let Some(item)  = existing_secondary_map.get(key){
                if item == &value {
                    id
                } else {
                    unimplemented!() // deeper check needed here;
                }
            } else {
                existing_secondary_map.insert(key, value);
                id
            }
        } else {

            self.register_secondarymap::<T>();
            self.get_mut_secondarymap::<T>().unwrap().insert(key,value);
            id

        }
    }

   
}
