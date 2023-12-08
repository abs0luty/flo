//! Input parsing logic for the `api` macro.

use crate::{asyncness::is_async, endpoint::EndpointAttr};

pub(crate) struct ApiMacroInput {
    pub(crate) api_trait_name: syn::Ident,
    pub(crate) methods: Vec<(syn::TraitItemFn, EndpointAttr)>,
}

pub(crate) fn parse_api_macro_input(
    item_trait: syn::ItemTrait,
) -> Result<ApiMacroInput, syn::Error> {
    let api_trait_name = item_trait.ident;
    let mut methods = vec![];

    // extract #[endpoint] attributes
    let mut trait_items_iter = item_trait.items.iter();
    while let Some(syn::TraitItem::Fn(method)) = trait_items_iter.next() {
        let Some(endpoint_attr_res) = EndpointAttr::from_attrs(&method.attrs) else {
            return Err(syn::Error::new_spanned(
                method,
                "#[endpoint] attribute is required",
            ));
        };
        let endpoint_attr = endpoint_attr_res?;

        if !is_async(&method.sig) {
            return Err(syn::Error::new_spanned(method, "method must be async"));
        }

        methods.push((method.clone(), endpoint_attr));
    }

    Ok(ApiMacroInput {
        api_trait_name,
        methods,
    })
}
