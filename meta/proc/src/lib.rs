//! Procedural macros.

use std::{iter::FilterMap, slice::Iter};

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Attribute, FnArg, Ident, ImplItem,
    ImplItemFn, ItemImpl, Pat, Signature, Type, TypePath,
};

/// TODO: Not sure if this is necessary yet, but we may want to include certain methods.
/// This would be the attribute ident added to each included function `#[wrap_include]`.
const _WRAP_INCLUDE_IDENT: &str = "wrap_include";
/// TODO: Not sure if this is necessary yet, but we may want to include certain methods.
/// This would be the attribute ident added to each excluded function `#[wrap_exclude]`.
const _WRAP_EXCLUDE_IDENT: &str = "wrap_exclude";

/// Intended for bindgen `impl` blocks.
///
/// Takes the entire `impl` blocks methods and generates those same methods for a
/// new wrapper type. This is useful because it removes the need to constantly use
/// the `unsafe` keyword for Reaper methods. It also an important piece of the
/// puzzle that gets us closer to a fully automated, self releasing crate.
#[proc_macro_attribute]
pub fn wrap_bindgen(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);

    // Get the name of the type that the methods are being implemented for
    let input_ident = if let Type::Path(TypePath { path, .. }) = *impl_block.self_ty.clone() {
        path.get_ident().unwrap().clone()
    } else {
        panic!("Unable to find Self Ident by path.");
    };

    let wrapper = generate_wrapper_impl(filter_impl_item_fns(&impl_block.items), &input_ident);

    // Combine the original impl block and the generated wrapper
    let expanded = quote! {
        #impl_block
        #wrapper
    };

    // Return the generated code as a TokenStream
    TokenStream::from(expanded)
}

/// Return true if the attribute path matches
/// a pattern which looks like `#[attribute_path]`.
fn _is_ident(attr: &Attribute, ident: &str) -> bool {
    attr.path().is_ident(ident)
}

type ImplItemFnFilterMap<'a> =
    FilterMap<Iter<'a, ImplItem>, fn(&'a ImplItem) -> Option<ImplItemFn>>;
/// Filter `ImplItemFn`s from the list of `ImplItem`s..
/// TODO: ..with a specified `#[attribute_path]`
fn filter_impl_item_fns(impl_items: &[ImplItem]) -> ImplItemFnFilterMap<'_> {
    impl_items.iter().filter_map(|item| -> Option<ImplItemFn> {
        if let ImplItem::Fn(method) = item {
            return Some(method.clone());
        }
        None
    })
}

fn check_signature(sig: &Signature) -> &Signature {
    if sig.unsafety.is_none() {
        panic!("wrap_bindgen proc macro expects unsafe methods")
    } else {
        sig
    }
}

type IdentFilterMap<'a> =
    FilterMap<syn::punctuated::Iter<'a, FnArg>, fn(&'a FnArg) -> Option<Ident>>;
/// Maps an iterator over the parameters to a function and returns
/// their `Ident`s (variable names).
fn filter_fn_arg_idents(parameters: &Punctuated<FnArg, Comma>) -> IdentFilterMap<'_> {
    parameters.iter().filter_map(|fn_arg| -> Option<Ident> {
        if let FnArg::Typed(pat_type) = &fn_arg {
            if let Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                return Some(pat_ident.ident);
            }
            return None;
        }
        None
    })
}

/// Creates a `TokenStream` from the inner type's `ImplItemFn`s that
/// will be passed to an `impl` block for the outer type.
fn generate_wrapper_method(method: ImplItemFn) -> proc_macro2::TokenStream {
    let sig: &Signature = check_signature(&method.sig);
    let method_name = &sig.ident;
    let parameters = &sig.inputs;
    let return_type = &sig.output;
    let args = filter_fn_arg_idents(parameters);

    quote! {
        pub fn #method_name(#parameters) #return_type {
            unsafe { (self.0).#method_name(#(#args),*) }
        }
    }
}

// TODO: figure out how to make an ident so this can be more dynamic
/// Generate the wrapper and impl methods for the wrapper.
fn generate_wrapper_impl<I>(methods: I, input_ident: &Ident) -> proc_macro2::TokenStream
where
    I: Iterator<Item = ImplItemFn>,
{
    let methods = methods.map(generate_wrapper_method);

    quote! {
        pub struct REAPER(#input_ident);
        impl REAPER { #(#methods)* }
    }
}
