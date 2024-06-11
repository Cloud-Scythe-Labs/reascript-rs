//! Procedural macros.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Pat, Signature,
    Type, TypePath,
};

/// Not sure if this is necessary yet, but we may want to include certain methods.
/// This would be the attribute ident added to each included function `#[wrap_include]`.
const _WRAP_INCLUDE_IDENT: &str = "wrap_include";
/// Not sure if this is necessary yet, but we may want to include certain methods.
/// This would be the attribute ident added to each excluded function `#[wrap_exclude]`.
const _WRAP_EXCLUDE_IDENT: &str = "wrap_exclude";

/// Intended for bindgen `impl` blocks. Takes the entire `impl` blocks methods
/// and generates those same methods for a new wrapper type. This is useful because
/// it removes the need to constantly use the `unsafe` keyword for Reaper methods.
/// It also an important piece of the puzzle that gets us closer to a fully automated,
/// self releasing crate.
///
/// NOTE: This is a WIP, and may not cover all cases, e.g. this assumes the methods are
/// `unsafe` and does not check before wrapping the inner method.
#[proc_macro_attribute]
pub fn wrap_bindgen(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);

    // Get the name of the type that the methods are being implemented for
    let input_ident = if let Type::Path(TypePath { path, .. }) = *impl_block.self_ty.clone() {
        path.get_ident().unwrap().clone()
    } else {
        // TODO: Add traits
        panic!("Unable to find Self type by path.");
    };

    let wrapper = generate_wrapper_impl(&extract_methods(&impl_block), &input_ident);

    // Return the generated code as a TokenStream
    TokenStream::from(wrapper)
}

/// Filter methods with a specified `#[attribute_path]`
fn extract_methods(impl_block: &ItemImpl) -> Vec<ImplItemFn> {
    impl_block
        .items
        .iter()
        .filter_map(|item| -> Option<ImplItemFn> {
            if let ImplItem::Fn(method) = item {
                // if method.attrs.iter().any(|attr| is_ident(attr, ident)) {
                return Some(method.clone());
                // }
            }
            None
        })
        .collect()
}

/// Return true if the attribute path matches
/// a pattern which looks like `#[attribute_path]`.
fn _is_ident(attr: &Attribute, ident: &str) -> bool {
    attr.path().is_ident(ident)
}

// TODO: figure out how to make an ident so this can be more dynamic
/// Generate the wrapper and impl methods for the wrapper.
fn generate_wrapper_impl(methods: &[ImplItemFn], input_ident: &Ident) -> proc_macro2::TokenStream {
    let methods = methods.iter().map(|method| {
        let sig: &Signature = &method.sig;
        let method_name = &sig.ident;
        let parameters = &sig.inputs;
        let return_type = &sig.output;
        let args = parameters.iter().filter_map(|fn_arg| -> Option<Ident> {
            if let FnArg::Typed(pat_type) = &fn_arg {
                if let Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                    Some(pat_ident.ident)
                } else {
                    None
                }
            } else {
                None
            }
        });

        quote! {
            pub fn #method_name(#parameters) #return_type {
                unsafe {
                    (self.0).#method_name(#(#args),*)
                }
            }
        }
    });

    quote! {
        pub struct REAPER(#input_ident);
        impl REAPER {
            #(#methods)*
        }
    }
}
