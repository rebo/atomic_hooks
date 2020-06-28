
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
    inverse : String,
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


    let template = input_fn.sig.inputs.iter().map(|_| "_{}").collect::<String>();


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
        template_quote = quote!(#arg_name_ident.to_string(),);
        use_args_quote = quote!(#arg_name_ident,);
        } else {
            first = false;
            template_quote = quote!(#template_quote,#arg_name_ident.to_string(),);
            use_args_quote = quote!(#use_args_quote, #arg_name_ident);
        }
    }

    let id_string_quote = quote!(format!(#template, #template_quote)); 
    
    let atom_default_ident = format_ident!("{}_with_default", atom_ident);
    
    let reset_atom_ident = format_ident!("reset_{}", atom_ident);
    

    let (atom_fn_ident,marker) = if args.undo {
        (format_ident!("atom_with_undo"), format_ident!("AllowUndo"))
    } else {
        (format_ident!("atom"), format_ident!("NoUndo"))
    };

    

    quote!(

        fn #atom_ident(#arg_quote) -> AtomStateAccess<#the_type,#marker,IsAnAtomState>{
            let atom_ident = format!("{}_{}",#atom_ident_string,#id_string_quote);
          
            #atom_fn_ident::<#the_type,_,#marker,IsAnAtomState>(&atom_ident,|| {
                #body         
            })
        } 

        fn #atom_default_ident<F:FnOnce() -> #the_type>( #arg_quote default : F ) -> AtomStateAccess<#the_type,#marker,IsAnAtomState>{
            let atom_ident = format!("{}_{}",#atom_ident_string,#id_string_quote);
            #atom_fn_ident::<#the_type,_,#marker,IsAnAtomState>(&atom_ident,default)
        } 

        fn #reset_atom_ident(#arg_quote){
            #atom_ident(#use_args_quote).update(|v| {
                 *v = {#body}   
                }
             );
         } 
  
        
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
pub fn reaction(_args: TokenStream, input: TokenStream) -> TokenStream {
   
    let input_fn: ItemFn = syn::parse_macro_input!(input);
    
    let input_fn_string = input_fn.sig.ident.to_string();
    let atom_ident_string = input_fn_string.as_str();

    let atom_ident = format_ident!("{}", atom_ident_string);

    let the_type = match input_fn.sig.output {
        syn::ReturnType::Default => panic!("Your atom MUST return a non-Unit value"),
        syn::ReturnType::Type(_, the_type) => the_type.clone(),
    };
    let body = input_fn.block.clone();

    
    let template = input_fn.sig.inputs.iter().map(|_| "_{}").collect::<String>();


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
        template_quote = quote!(#arg_name_ident.to_string(),);
        } else {
            first = false;
            template_quote = quote!(#template_quote,#arg_name_ident.to_string(),);
        }
    }
    

    let id_string_quote = quote!(format!(#template, #template_quote)); 
    let quote = quote!(

        fn #atom_ident(#arg_quote) -> AtomStateAccess<#the_type,NoUndo,IsAReactionState>{
            let atom_ident = format!("{}_{}",#atom_ident_string,#id_string_quote);
           
           
            if !atom_state_exists_for_id::<#the_type>(&atom_ident){
            let atom_ident2 = atom_ident.clone();
            
            
            let func = move |_stre: &str| {
                let getter = Getter::new(&atom_ident2.clone());
                illicit::child_env!( std::cell::RefCell<Getter> => std::cell::RefCell::new(getter) ).enter(|| {
                    // let mut existing_state = remove_atom_state_with_id::<#the_type>(&atom_ident2.clone());
                    let value = {#body};
                    set_atom_state_with_id::<#the_type>(value,&atom_ident2.clone());
                    // we need to remove dependencies that do nto exist anymore
                    unlink_dead_links(&atom_ident2.clone());
                })
            };

            // func(&atom_ident);

            reaction::<#the_type,NoUndo,IsAReactionState,_>(&atom_ident.clone(),func)
            } else {
                AtomStateAccess::<#the_type,NoUndo,IsAReactionState>::new(&atom_ident)                 
            }
        }
    );

    quote.into()
}



#[proc_macro_attribute]
pub fn set_reaction(_args: TokenStream, input: TokenStream) -> TokenStream {
    
    let input_fn: ItemFn = syn::parse_macro_input!(input);

    let input_fn_string = input_fn.sig.ident.to_string();
    let atom_ident_string = input_fn_string.as_str();

    let atom_ident = format_ident!("{}", atom_ident_string);

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
    let mut use_args_quote = quote!();

    let mut first = true;
    for input in inputs_iter_2 {
        let arg_name_ident = format_ident!("{}",get_arg_name(input));
        
        
        if first {
        template_quote = quote!(#arg_name_ident.to_string(),);
        use_args_quote = quote!(#arg_name_ident,);
        } else {
            first = false;
            template_quote = quote!(#template_quote,#arg_name_ident.to_string(),);
            use_args_quote = quote!(#use_args_quote, #arg_name_ident);
        }
    }

    
    
    
    
    let reverse_atom_ident = format_ident!("set_{}", atom_ident);

    // let (atom_fn_ident,marker) = if args.undo {
    //     (format_ident!("atom_with_undo"), format_ident!("AllowUndo"))
    // } else {
    //     (format_ident!("atom"), format_ident!("NoUndo"))
    // };

    

    quote!(
    fn #reverse_atom_ident(#arg_quote){
        #atom_ident(#use_args_quote).get_with(|value| {
            #body
        } )
    }   

    ).into()

}
