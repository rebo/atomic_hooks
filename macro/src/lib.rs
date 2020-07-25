
extern crate darling;
extern crate proc_macro;
use darling::FromMeta;
use self::proc_macro::TokenStream;

use quote::{format_ident, quote};
// use syn::parse::{Parse, ParseStream, Result};
// use syn::{parse_macro_input, DeriveInput, Expr, ExprArray};
 use syn::{FnArg, ItemFn, Pat};
// use syn::{Lit, Meta, MetaNameValue};
use syn::{AttributeArgs};


#[derive(Debug, FromMeta)]
struct MacroArgs {
    #[darling(default)]
    undo: bool,
}

#[derive(Debug, FromMeta)]
struct ReactionMacroArgs {
    #[darling(default)]
    existing_state: bool,
    #[darling(default)]
    suspended:bool,
}


#[proc_macro_attribute]
pub fn atom(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = syn::parse_macro_input!(args as AttributeArgs);   

    let input_fn: ItemFn = syn::parse_macro_input!(input);
    let vis = input_fn.vis.clone();

    


    let args = match MacroArgs::from_list(&attr_args){
        Ok(v) => v,
        Err(e) => panic!("{}",e),
    };

    

    let atom_fn_ident = if args.undo {
        format_ident!("atom_undo")
    }  else {
        format_ident!("atom")
    };
    
    let sig = input_fn.sig.clone();
    

    let the_outer_type = match input_fn.sig.output {
        syn::ReturnType::Default => panic!("Your atom MUST return a non-Unit value"),
        syn::ReturnType::Type(_, the_type) => the_type.clone(),
    };


    let the_type = 
    if args.undo {
        match *the_outer_type {
            syn::Type::Path(p) => {
                
                if let Some(atom_segment) = p.path.segments.first() {
                    if atom_segment.ident.to_string() != "AtomUndo" {
                        panic!("You really need to return an AtomUndo wrapped type");
                    }
                    match &atom_segment.arguments {
                            syn::PathArguments::AngleBracketed(angle_brack_args) => {
                                let first_arg = angle_brack_args.args.first().expect("AtomUndo should have a first type");
                                if let syn::GenericArgument::Type(a_type) = first_arg {
                                    a_type.clone()
                                } else {
                                    panic!("AtomUndo doest hold a type")
                                }
                            },
                            _ => panic!("AtomUndo has no type???")
                    }
                } else { 
                    panic!("You do need to return an AtomUndo wrapped type");
                }
            },
            _ => panic!("You need to return an AtomUndo wrapped type"),
        }

    } else {
    match *the_outer_type {
        syn::Type::Path(p) => {
            
            if let Some(atom_segment) = p.path.segments.first() {
                if atom_segment.ident.to_string() != "Atom" {
                    panic!("You really need to return an atom wrapped type");
                }
                match &atom_segment.arguments {
                        syn::PathArguments::AngleBracketed(angle_brack_args) => {
                            let first_arg = angle_brack_args.args.first().expect("atom should have a first type");
                            if let syn::GenericArgument::Type(a_type) = first_arg {
                                a_type.clone()
                            } else {
                                panic!("atom doest hold a type")
                            }
                        },
                        _ => panic!("Atom has no type???")
                }
            } else { 
                panic!("You do need to return an atom wrapped type");
            }
        },
        _ => panic!("You need to return an atom wrapped type"),
    }
};

    let body = input_fn.block.clone();


    let inputs_iter = &mut input_fn.sig.inputs.iter();
    let  mut inputs_iter_3 = inputs_iter.clone();

    let  inputs_iter_2 = inputs_iter.clone();
    
    
    let mut arg_quote;
    if let Some(first_arg) = inputs_iter_3.next(){
        arg_quote  = quote!(#first_arg,);
        for input in inputs_iter_3 {
            arg_quote = quote!(#arg_quote, #input,);
        }
    }   
    
    let mut template_quote = quote!();
    let mut use_args_quote = quote!();

    let mut first = true;
    for input in inputs_iter_2 {
        let arg_name_ident = format_ident!("{}",get_arg_name(input));
        
        
        if first {
        template_quote = quote!(#arg_name_ident.clone(),);
        use_args_quote = quote!(let #arg_name_ident = #arg_name_ident.clone(););
        
        first = false;    
        } else {
            template_quote = quote!(#template_quote #arg_name_ident.clone(),);
            use_args_quote = quote!(#use_args_quote let #arg_name_ident = #arg_name_ident.clone(););
        }
    }

    let hash_quote = quote!( (CallSite::here(), #template_quote) );
    


    let set_inert_with_undo= if args.undo {
        quote!( set_inert_atom_state_with_id_with_undo::<#the_type>(value,__id ); )
     } else {
        quote!( set_inert_atom_state_with_id::<#the_type>(value,__id );)
     };


    
    quote!(

       #vis #sig{

                let __id  = return_key_for_type_and_insert_if_required(#hash_quote);

                let func = move || {
                    #use_args_quote

                    topo::call(||{
                        
                        illicit::Env::hide::<topo::Point>();
                        topo::call(||{
                            
                            let context = ReactiveContext::new(__id );
                            illicit::child_env!( std::cell::RefCell<ReactiveContext> => std::cell::RefCell::new(context) ).enter(|| {
                                
                                let value = {#body};
                                #set_inert_with_undo
                            })


                        })
                    })
                };

                #atom_fn_ident::<#the_type,_>(__id ,func)
            
        } 

        // fn #atom_default_ident<F:FnOnce() -> #the_type>( #arg_quote default : F ) -> ReactiveStateAccess<#the_type,#marker,IsAnAtomState>{
        //     let atom_ident = format!(#atom_atom , module_path!(), #template_quote);

        //     #atom_fn_ident::<#the_type,_,#marker,IsAnAtomState>(&atom_ident,default)
        // } 

        // fn #reset_atom_ident(#arg_quote){
        //     illicit::Env::hide::<topo::Point>();
        //     topo::call(||{
        //         #atom_ident(#use_args_quote).update(|v| {
        //             *v = {#body}   
        //             }
        //         );
        //     })
        //  } 
  
        
    ).into()

}


fn get_arg_name(fnarg : &FnArg) -> String {
    match fnarg {
            FnArg::Receiver(_) => panic!("cannot be a method with self receiver"),
            FnArg::Typed(t) => {
                match &*t.pat {
                    Pat::Ident(syn::PatIdent { ident, .. }) => ident.to_string(), //syn::parse_quote!(&#ident),
                    _ => unimplemented!("Cannot get arg name"),
                }
            }
    }
}


#[proc_macro_attribute]
pub fn reaction(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = syn::parse_macro_input!(args as AttributeArgs);

    let args = match ReactionMacroArgs::from_list(&attr_args){
        Ok(v) => v,
        Err(e) => panic!("{}",e),
    };

    let reaction_suspended_ident = if args.suspended {
        format_ident!("reaction_start_suspended")
    } else {
        format_ident!("reaction")
    };
    
    let input_fn: ItemFn = syn::parse_macro_input!(input);
    
    

    
    let sig = input_fn.sig.clone();
    let vis = input_fn.vis.clone();

    let the_outer_type = match input_fn.sig.output.clone() {
        syn::ReturnType::Default => panic!("Your atom MUST return a non-Unit value"),
        syn::ReturnType::Type(_, the_type) => the_type.clone(),
    };



    let the_type = match *the_outer_type {
        syn::Type::Path(p) => {
            
            if let Some(atom_segment) = p.path.segments.first() {
                if atom_segment.ident.to_string() != "Reaction" {
                    panic!("You really need to return an Reaction wrapped type");
                }
                match &atom_segment.arguments {
                        syn::PathArguments::AngleBracketed(angle_brack_args) => {
                            let first_arg = angle_brack_args.args.first().expect("Reaction should have a first type");
                            if let syn::GenericArgument::Type(a_type) = first_arg {
                                a_type.clone()
                            } else {
                                panic!("Reaction doest hold a type")
                            }
                        },
                        _ => panic!("Reaction has no type???")
                }
            } else { 
                panic!("You do need to return an Reaction wrapped type");
            }
        },
        _ => panic!("You need to return an Reaction wrapped type"),

    };



    let body = input_fn.block.clone();

    let inputs_iter = &mut input_fn.sig.inputs.iter();
    let  mut inputs_iter_3 = inputs_iter.clone();

    let  inputs_iter_2 = inputs_iter.clone();
    
    
    let mut arg_quote;
    if let Some(first_arg) = inputs_iter_3.next(){
        arg_quote  = quote!(#first_arg);
        for input in inputs_iter_3 {
            arg_quote = quote!(#arg_quote, #input);
        }
    }   
    
    let mut template_quote = quote!();
    let mut use_args_quote = quote!();

    let mut first = true;
    for input in inputs_iter_2 {
        let arg_name_ident = format_ident!("{}",get_arg_name(input));
        
        
        if first {
        template_quote = quote!(#arg_name_ident.clone(),);
        use_args_quote = quote!(let #arg_name_ident = #arg_name_ident.clone(););
        
        first = false;
        } else {
            template_quote = quote!(#template_quote #arg_name_ident.clone(),);
            use_args_quote = quote!(#use_args_quote let #arg_name_ident = #arg_name_ident.clone(););
        }
    }

    let hash_quote = quote!( (CallSite::here(), #template_quote) );

    let use_existing_state = if args.existing_state {
        quote!(
            let mut existing_state = clone_reactive_state_with_id::<#the_type>(__id);
        )
    } else {
        quote!()
    };

    
    let quote = 
        quote!(

            #vis #sig{


                    let __id = return_key_for_type_and_insert_if_required(#hash_quote);

               
                    if !reactive_state_exists_for_id::<#the_type>(__id ){
               
                        let func = move || {
                            #use_args_quote
                            
                            topo::call(||{
                                
                            illicit::Env::hide::<topo::Point>();
                            
                            topo::call(||{
                                
                            let mut _context = ReactiveContext::new(__id );
                            {
                            
                            illicit::child_env!( std::cell::RefCell<ReactiveContext> => std::cell::RefCell::new(_context) ).enter(|| {
                                
                                
                                #use_existing_state
                                let value = {#body};
                                set_inert_atom_state_with_id::<#the_type>(value,__id );
                                // we need to remove dependencies that do nto exist anymore
                                unlink_dead_links(__id );
                            })
                            
                        }
                        })
                        
                    })
                    


                        };
                        

                        #reaction_suspended_ident::<#the_type,_>(__id ,func)
                    } else {
                        Reaction::<#the_type>::new(__id )                 
                    }
                
            }
            
        );
    

    quote.into()
}

