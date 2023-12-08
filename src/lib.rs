#![doc = include_str!("../README.md")]

use crate::input::ApiMacroInput;
use expand::expand_api_macro;
use input::parse_api_macro_input;
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod asyncness;
mod endpoint;
mod expand;
mod input;
mod remove_attrs;

/// See [crate level documentation] for details.
///
/// [crate level documentation]: crate
#[proc_macro_attribute]
pub fn api(attr: TokenStream, item: TokenStream) -> TokenStream {
    let api_client_struct_name = parse_macro_input!(attr as syn::Ident);
    let item_trait = parse_macro_input!(item as syn::ItemTrait);
    let ApiMacroInput {
        api_trait_name,
        methods,
    } = match parse_api_macro_input(item_trait.clone()) {
        Ok(input) => input,
        Err(err) => return err.into_compile_error().into(),
    };

    expand_api_macro(api_client_struct_name, api_trait_name, methods, item_trait).into()
}
