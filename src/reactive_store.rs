use anymap::any::Any;
use slotmap::{DenseSlotMap,DefaultKey, Key, SecondaryMap};
use std::collections::HashMap;
// use seed::*;
use std::rc::Rc;


#[derive(Debug,Clone)]
pub struct Context {
    
    pub reaction_key: String,
    pub reactive_state_accessors: Vec<String>,
}

impl Context {
    pub fn new(reaction_key : &str) -> Context {
        Context{
            reaction_key: reaction_key.to_string(),
            reactive_state_accessors:vec![],
        }
    }
}


#[derive(Clone)]
pub struct Reaction {
    pub func: Rc<dyn Fn() -> () + 'static>
}

pub struct AtomStore {
    pub id_to_key_map: HashMap<String, DefaultKey>,
    pub primary_slotmap: DenseSlotMap<DefaultKey, String>,
    pub anymap: anymap::Map<dyn Any>,
    // pub reaction_graph: DenseSlotMap<DefaultKey, StoreNode>,
    
    // pub reaction_funcs: HashMap<String, Box<dyn CalcReaction>>,
}

impl AtomStore {
    pub(crate) fn new() -> AtomStore {
        AtomStore {
            id_to_key_map: HashMap::new(),
            primary_slotmap: DenseSlotMap::new(),
            anymap: anymap::Map::new(),
            
            // reaction_graph: DenseSlotMap::new(),
            
            // reaction_funcs: HashMap::new(), // this probably needs to be a secondary map
        }
    }

//     println!("adding dep, {} {}", dep , reaction_key);
//     let dep_sm_key = if let Some(dep_sm_key) = self.id_to_key_map.get(dep){
//         dep_sm_key.clone()


//     } else {
//         panic!("adding dep, {} {}", dep , reaction_key);
//     };
//     let reaction_sm_key = if let Some(reaction_sm_key)  = self.id_to_key_map.get(reaction_key){
//         reaction_sm_key.clone()
//     } else {
//         panic!("adddding dep, {} {}", dep , reaction_key);
// };



pub fn new_reaction(&mut self, reaction_sm_key: &str, func: Reaction){
    
    let key = self.id_to_key_map.get(reaction_sm_key).unwrap().clone();
    if let Some(map) = self.get_mut_secondarymap::<Reaction>(){
        
        map.insert(key, func);
        
    } else {
        let mut sm: SecondaryMap<DefaultKey, Reaction> = SecondaryMap::new();
        sm.insert(key, func);
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

    pub fn remove_dependency(&mut self,source_id:&str, reaction_id:&str){

  
        // println!("adding dep, {} {}", dep , reaction_key);
        let source_sm_key = self.id_to_key_map.get(source_id).unwrap().clone();
        let reaction_sm_key = self.id_to_key_map.get(reaction_id).unwrap().clone();
        
        // if dep == "todo_input_state_" && reaction_key =="add_todo_" {
        //     panic!("cannot find {:#?} for id {} and reaction_key {:#?}",dep, reaction_key, reaction_sm_key);
        //     }

        let map = &mut self.get_mut_secondarymap::<Vec<DefaultKey>>().unwrap();
        
            if let Some(nodes) = map.get_mut(source_sm_key) {
                nodes.retain(|n| *n != reaction_sm_key);
            } else {
                panic!("Trying to remove a from a state which does not exit")
            }
    }


        // // println!("adding dep, {} {}", dep , reaction_key);
        // let source_sm_key = self.id_to_key_map.get(source_id).unwrap().clone();
        // let reaction_sm_key = self.id_to_key_map.get(reaction_id).unwrap().clone();
        
        // // if dep == "todo_input_state_" && reaction_key =="add_todo_" {
        // //     panic!("cannot find {:#?} for id {} and reaction_key {:#?}",dep, reaction_key, reaction_sm_key);
        // //     }

        // let map = &mut self.get_mut_secondarymap::<Vec<DefaultKey>>().unwrap();
        
        //     if let Some(nodes) = map.get_mut(source_sm_key) {
        //         if !nodes.contains(&reaction_sm_key) {
        //             nodes.push(reaction_sm_key)
        //         }
        //     } else {
        //         map.insert(source_sm_key, vec![reaction_sm_key]);
        //     }
            
        
        

        // if let Some(graph_node) = self.reaction_graph.get_mut(dep)


        // let dependency = self.remove_state_with_id(current_id)

        // let dep_key = self.reaction_id_to_key_map.get(dep).unwrap();
        

        // let reaction_store_node =  StoreNode::Reaction(reaction);
            
        // set_state_with_id::<StoreNode>()(  reaction_data_store_node, dep);


        // if let Some(reaction_key) = self.reaction_id_to_key_map.get(reaction) {
        //     if let Some(entry) = self.reaction_graph.get_mut(*reaction_key){
        //     entry.1.push(*dep_key);
        //     }
        // } else {
        //     let reaction_key = self.reaction_graph.insert(
        //         (reaction.to_string(), vec![])
        //     );
        //     self.reaction_id_to_key_map.insert(reaction.to_string(), reaction_key);
        // }

    // }



    pub fn add_dependency(&mut self,source_id:&str, reaction_id:&str){
        
       
        // println!("adding dep, {} {}", dep , reaction_key);
        let source_sm_key = self.id_to_key_map.get(source_id).unwrap().clone();
        let reaction_sm_key = self.id_to_key_map.get(reaction_id).unwrap().clone();
        
        // if dep == "todo_input_state_" && reaction_key =="add_todo_" {
        //     panic!("cannot find {:#?} for id {} and reaction_key {:#?}",dep, reaction_key, reaction_sm_key);
        //     }

        let map = &mut self.get_mut_secondarymap::<Vec<DefaultKey>>().unwrap();
        
            if let Some(nodes) = map.get_mut(source_sm_key) {
                if !nodes.contains(&reaction_sm_key) {
                    nodes.push(reaction_sm_key)
                }
            } else {
                map.insert(source_sm_key, vec![reaction_sm_key]);
            }
            
        
        

        // if let Some(graph_node) = self.reaction_graph.get_mut(dep)


        // let dependency = self.remove_state_with_id(current_id)

        // let dep_key = self.reaction_id_to_key_map.get(dep).unwrap();
        

        // let reaction_store_node =  StoreNode::Reaction(reaction);
            
        // set_state_with_id::<StoreNode>()(  reaction_data_store_node, dep);


        // if let Some(reaction_key) = self.reaction_id_to_key_map.get(reaction) {
        //     if let Some(entry) = self.reaction_graph.get_mut(*reaction_key){
        //     entry.1.push(*dep_key);
        //     }
        // } else {
        //     let reaction_key = self.reaction_graph.insert(
        //         (reaction.to_string(), vec![])
        //     );
        //     self.reaction_id_to_key_map.insert(reaction.to_string(), reaction_key);
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

    pub fn get_state_with_id<T: 'static>(
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

    pub(crate) fn clone_dep_funcs_for_id(&mut self, id: &str)-> Vec<(String, Reaction )>{
        
        // let reaction_keys  = if let Some(reaction_keys) = self.get_state_with_id::<Vec<DefaultKey>>(id){
        //     reaction_keys.clone()
        // } else {
        //     vec![]
        // };

        let  reaction_keys = self.get_state_with_id::<Vec<DefaultKey>>(id).cloned();
        
         if let Some(reaction_keys) = &reaction_keys {
        
        reaction_keys.iter().filter_map(|key|  {
            
            if let Some(existing_secondary_map) = self.get_mut_secondarymap::<Reaction>() {
                
                if let Some( reaction) =  existing_secondary_map.get(*key).cloned(){
                
               Some((self.primary_slotmap.get(*key).unwrap().clone(),reaction))
                } else {
                    panic!("cannot find {:#?} for id {}",key, id);
                }
            } else {
                None
            }
        }).collect::<Vec<(String,Reaction)>>()
        }
        else {
            vec![]
        }    
    
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

    pub fn get_mut_secondarymap<T: 'static>(&mut self) -> Option<&mut SecondaryMap<DefaultKey, T>> {
        self.anymap.get_mut::<SecondaryMap<DefaultKey, T>>()
    }

    pub fn register_secondarymap<T: 'static>(&mut self) {
        let sm: SecondaryMap<DefaultKey, T> = SecondaryMap::new();
        self.anymap.insert(sm);
    }
}
