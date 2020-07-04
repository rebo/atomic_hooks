
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
    always_run : bool,
}

#[proc_macro_attribute]
pub fn atom(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = syn::parse_macro_input!(args as AttributeArgs);

    let args = match MacroArgs::from_list(&attr_args){
        Ok(v) => v,
        Err(e) => panic!("{}",e),
    };

    let input_fn: ItemFn = syn::parse_macro_input!(input);

    let input_fn_string = input_fn.sig.ident.to_string();
    let atom_ident_string = input_fn_string.as_str();

    let atom_ident = format_ident!("{}", atom_ident_string);

    let the_type = match input_fn.sig.output {
        syn::ReturnType::Default => panic!("Your atom MUST return a non-Unit value"),
        syn::ReturnType::Type(_, the_type) => the_type.clone(),
    };
    let body = input_fn.block.clone();


    


    let inputs_iter = &mut input_fn.sig.inputs.iter();
    let  mut inputs_iter_3 = inputs_iter.clone();

    let  inputs_iter_2 = inputs_iter.clone();
    
    
    let mut arg_quote = quote!();
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
        template_quote = quote!(#arg_name_ident,);
        use_args_quote = quote!(#arg_name_ident,);
        } else {
            first = false;
            template_quote = quote!(#template_quote,#arg_name_ident,);
            use_args_quote = quote!(#use_args_quote, #arg_name_ident);
        }
    }

    let hash_quote = quote!( (CallSite::here(), #template_quote) );
    

    let (atom_fn_ident,marker) = if args.undo {
        (format_ident!("atom_with_undo"), format_ident!("AllowUndo"))
    } else {
        (format_ident!("atom"), format_ident!("NoUndo"))
    };


    let update_with_undo= if args.undo {
       quote!( set_inert_atom_state_with_id(UndoVec::<#the_type>(vec![value]), &__id); )
    } else {
       quote!()
    };

    let set_inert_with_undo= if args.undo {
        quote!( set_inert_atom_state_with_id::<#the_type>(value.clone(),&__id ); )
     } else {
        quote!(set_inert_atom_state_with_id::<#the_type>(value,&__id );)
     };


    
    quote!(

        pub fn #atom_ident(#arg_quote) -> ReactiveStateAccess<#the_type,#marker,IsAnAtomState>{
                let __id  = return_key_for_type_and_insert_if_required(#hash_quote);
                let func = move || {
                    topo::call(||{
                        illicit::Env::hide::<topo::Point>();
                        topo::call(||{

                            let context = ReactiveContext::new(__id );
                            illicit::child_env!( std::cell::RefCell<ReactiveContext> => std::cell::RefCell::new(context) ).enter(|| {

                                let value = {#body};
                                #set_inert_with_undo
                                #update_with_undo
                            })


                        })
                    })
                };

                #atom_fn_ident::<#the_type,_,#marker,IsAnAtomState>(__id ,func)
            
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


    let input_fn: ItemFn = syn::parse_macro_input!(input);
    
    let input_fn_string = input_fn.sig.ident.to_string();
    let atom_ident_string = input_fn_string.as_str();

    let atom_ident = format_ident!("{}", atom_ident_string);

    let the_type = match input_fn.sig.output {
        syn::ReturnType::Default => panic!("Your atom MUST return a non-Unit value"),
        syn::ReturnType::Type(_, the_type) => the_type.clone(),
    };
    let body = input_fn.block.clone();

    let inputs_iter = &mut input_fn.sig.inputs.iter();
    let  mut inputs_iter_3 = inputs_iter.clone();

    let  inputs_iter_2 = inputs_iter.clone();
    
    
    let mut arg_quote = quote!();
    if let Some(first_arg) = inputs_iter_3.next(){
        arg_quote  = quote!(#first_arg);
        for input in inputs_iter_3 {
            arg_quote = quote!(#arg_quote, #input);
        }
    }   
    
    let mut template_quote = quote!();

    let mut first = true;
    for input in inputs_iter_2 {
        let arg_name_ident = format_ident!("{}",get_arg_name(input));
        if first {
        template_quote = quote!(#arg_name_ident,);
        } else {
            first = false;
            template_quote = quote!(#template_quote,#arg_name_ident,);
        }
    }
    let hash_quote = quote!( (CallSite::here(), #template_quote) );

    let always_run_quote =  if args.always_run { quote!(_context.always_run = true;) } else {quote!()};

    let quote = quote!(

        pub fn #atom_ident<'_a>(#arg_quote) -> ReactiveStateAccess<#the_type,NoUndo,IsAReactionState>{
          
                let __id = return_key_for_type_and_insert_if_required(#hash_quote);
            
                if !reactive_state_exists_for_id::<#the_type>(&__id ){
                                    
                    let func = move || {
                        topo::call(||{
                        illicit::Env::hide::<topo::Point>();
                        topo::call(||{


                        let mut _context = ReactiveContext::new(__id );
                        
                        #always_run_quote

                        illicit::child_env!( std::cell::RefCell<ReactiveContext> => std::cell::RefCell::new(_context) ).enter(|| {
                            // let mut existing_state = remove_reactive_state_with_id::<#the_type>(&atom_ident2.clone());
                            let value = {#body};
                            set_inert_atom_state_with_id::<#the_type>(value,&__id );
                            // we need to remove dependencies that do nto exist anymore
                            unlink_dead_links(&__id );
                        })
                    })
                })


                    };

                    reaction::<#the_type,NoUndo,IsAReactionState,_>(__id ,func)
                } else {
                    ReactiveStateAccess::<#the_type,NoUndo,IsAReactionState>::new(__id )                 
                }
            
        }
        
    );

    quote.into()
}



