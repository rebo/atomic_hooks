use std::marker::PhantomData;
use anymap::any::Any;
use slotmap::{DenseSlotMap,DefaultKey, Key, SecondaryMap};
// use std::collections::hash_map::DefaultHasher;
// use std::collections::HashSet;
// use std::hash::{Hash, Hasher};
use std::collections::HashMap;

#[derive(Debug)]

pub struct Getter{
    pub selector_key: String,
    pub atomic_state_accessors: Vec<String>,
}

impl Getter {
    pub fn new(selector_key : &str) -> Getter {
        Getter{
            selector_key: selector_key.to_string(),
            atomic_state_accessors:vec![],
        }
    }
}




pub struct Selector<T,F> 
    where F: FnOnce() 
{
    pub func: F,
    pub _phantom_data: PhantomData<T>,

}

pub trait CalcSelector {
    fn calc(&self);
}
impl <T,F> CalcSelector for Selector<T,F>  
    where F: FnOnce() -> () + Clone
{
    fn calc(&self){
        (self.func.clone())();
    }
}


pub struct AtomicStore {
    pub id_to_key_map: HashMap<String, DefaultKey>,
    pub primary_slotmap: DenseSlotMap<DefaultKey, String>,
    pub anymap: anymap::Map<dyn Any>,
    // pub selector_graph: DenseSlotMap<DefaultKey, StoreNode>,
    
    // pub selector_funcs: HashMap<String, Box<dyn CalcSelector>>,
}

impl AtomicStore {
    pub(crate) fn new() -> AtomicStore {
        AtomicStore {
            id_to_key_map: HashMap::new(),
            primary_slotmap: DenseSlotMap::new(),
            anymap: anymap::Map::new(),
            
            // selector_graph: DenseSlotMap::new(),
            
            // selector_funcs: HashMap::new(), // this probably needs to be a secondary map
        }
    }

pub fn new_selector(&mut self, selector_sm_key: DefaultKey, func: Box<dyn CalcSelector>){

    // let dep_sm_key = self.selector_id_to_key_map.get(dep).unwrap();
    


    if let Some(map) = self.get_mut_secondarymap::<Box<dyn CalcSelector>>(){
        map.insert(selector_sm_key, func);
        
    } else {
        let mut sm: SecondaryMap<DefaultKey, Box<dyn CalcSelector>> = SecondaryMap::new();
        sm.insert(selector_sm_key, func);
        self.anymap.insert(sm);
    }

}

    pub fn add_atom(&mut self, id: &str) {
        let dep_sm_key = self.id_to_key_map.get(id).unwrap().clone();

        if self.get_secondarymap::<Vec<DefaultKey>>().is_none(){
            self.register_secondarymap::<Vec<DefaultKey>>();
        }

        let map = &mut self.get_mut_secondarymap::<Vec<DefaultKey>>().unwrap();
        map.insert(dep_sm_key, vec![]);
    }


    pub fn add_dependency(&mut self,dep:&str, selector_key:&str){
        // println!("adding dep, {} {}", dep , selector_key);
        let dep_sm_key = self.id_to_key_map.get(dep).unwrap().clone();
        let selector_sm_key = self.id_to_key_map.get(selector_key).unwrap().clone();

        if let Some(map) = &mut self.get_mut_secondarymap::<Vec<DefaultKey>>(){
            if let Some(nodes) = map.get_mut(dep_sm_key) {
                if !nodes.contains(&selector_sm_key) {
                    nodes.push(selector_sm_key)
                }
                } else {
                    map.insert(dep_sm_key, vec![selector_sm_key]);
                }
            // println!("{:#?}",map);
        } else {
            self.register_secondarymap::<Vec<DefaultKey>>();
        }

        // if let Some(graph_node) = self.selector_graph.get_mut(dep)


        // let dependency = self.remove_state_with_id(current_id)

        // let dep_key = self.selector_id_to_key_map.get(dep).unwrap();
        

        // let selector_store_node =  StoreNode::Selector(selector);
            
        // set_state_with_id::<StoreNode>()(  selector_data_store_node, dep);


        // if let Some(selector_key) = self.selector_id_to_key_map.get(selector) {
        //     if let Some(entry) = self.selector_graph.get_mut(*selector_key){
        //     entry.1.push(*dep_key);
        //     }
        // } else {
        //     let selector_key = self.selector_graph.insert(
        //         (selector.to_string(), vec![])
        //     );
        //     self.selector_id_to_key_map.insert(selector.to_string(), selector_key);
        // }

    }

    pub(crate) fn state_exists_with_id<T: 'static>(&self, id: &str) -> bool {
        match (self.id_to_key_map.get(id), self.get_secondarymap::<T>()) {
            (Some(existing_key), Some(existing_secondary_map)) => {
                existing_secondary_map.contains_key(*existing_key)
            }
            (_, _) => false,
        }
    }

    pub(crate) fn get_state_with_id<T: 'static>(
        &self,
        current_id: &str,
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
        current_id: &str,
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

    // pub(crate) fn remove_topo_id(&mut self, id: topo::Id) {
    //     let key = self.id_to_key_map.get(&id).copied().unwrap_or_default();
    //     if !key.is_null() {
    //         self.primary_slotmap.remove(key);
    //         self.id_to_key_map.remove(&id);
    //     }
    // }

    pub(crate) fn clone_dep_funcs_for_id(&mut self, id: &str)-> Vec<(String, Box<dyn CalcSelector + 'static> )>{

        let selector_keys  = if let Some(selector_keys) = self.get_state_with_id::<Vec<DefaultKey>>(id){
            selector_keys.clone()
        } else {
            vec![]
        };

    
        selector_keys.iter().filter_map(|key|  {
            let id = self.primary_slotmap.get(*key).cloned().unwrap();
            if let Some(existing_secondary_map) = self.get_mut_secondarymap::<Box<dyn CalcSelector>>() {
                
                // log!("aboute to remove {}",id);
               Some((id.to_string(), existing_secondary_map.remove(*key).unwrap()))
            } else {
                None
            }
        }).collect::<Vec<(String,Box<dyn CalcSelector>)>>()

    }

    pub(crate) fn set_state_with_id<T: 'static>(&mut self, data: T, current_id: &str) {

        
        //unwrap or default to keep borrow checker happy
        let key = self
            .id_to_key_map
            .get(current_id)
            .copied()
            .unwrap_or_default();

        // println!("{:#?}", key);

        if key.is_null() {
            let key = self.primary_slotmap.insert(current_id.to_string());
            self.id_to_key_map.insert(current_id.to_string(), key);
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

    fn get_mut_secondarymap<T: 'static>(&mut self) -> Option<&mut SecondaryMap<DefaultKey, T>> {
        self.anymap.get_mut::<SecondaryMap<DefaultKey, T>>()
    }

    pub fn register_secondarymap<T: 'static>(&mut self) {
        let sm: SecondaryMap<DefaultKey, T> = SecondaryMap::new();
        self.anymap.insert(sm);
    }
}
