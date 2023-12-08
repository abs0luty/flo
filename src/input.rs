//! Input parsing logic for the `api` macro.

use syn::{Error, Ident, ItemTrait, TraitItem, TraitItemFn};

use crate::{asyncness::is_async, endpoint::EndpointAttr};

pub(crate) struct ApiMacroInput {
    pub(crate) api_trait_name: Ident,
    pub(crate) methods: Vec<(TraitItemFn, EndpointAttr)>,
}

pub(crate) fn parse_api_macro_input(item_trait: ItemTrait) -> Result<ApiMacroInput, Error> {
    let api_trait_name = item_trait.ident;
    let mut methods = vec![];

    // Extract endpoint attributes from the trait methods.
    let mut trait_items_iter = item_trait.items.iter();
    while let Some(TraitItem::Fn(method)) = trait_items_iter.next() {
        let Some(endpoint_attr_res) = EndpointAttr::from_attrs(&method.attrs) else {
            return Err(Error::new_spanned(
                method,
                "one of: #[get(...)], #[post(...)], #[put(...)], #[delete(...)], #[head(...)] or
#[patch(...)] endpoint attributes is required",
            ));
        };
        let endpoint_attr = endpoint_attr_res?;

        if !is_async(&method.sig) {
            return Err(Error::new_spanned(method, "method must be async"));
        }

        methods.push((method.clone(), endpoint_attr));
    }

    Ok(ApiMacroInput {
        api_trait_name,
        methods,
    })
}
