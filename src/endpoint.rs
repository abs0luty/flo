//! Defines a parser for `#[get(...)], #[post(...)]` and other
//! endpoint attributes.

use paste::paste;

macro_rules! generate_endpoint_attr_kind {
    ($($method_name:ident),*) => {
        paste! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub(crate) enum HttpMethod {
                $(
                    [<$method_name:camel>]
                ),*
            }

            impl HttpMethod {
                pub(crate) fn snake_case_ident(&self) -> proc_macro2::Ident {
                    proc_macro2::Ident::new(
                        &self.name(),
                        proc_macro2::Span::call_site()
                    )
                }

                /// Returns method name in snake case.
                pub(crate) fn name(&self) -> &'static str {
                    match self {
                        $(Self::[<$method_name:camel>] => stringify!($method_name)),*
                    }
                }
            }

            pub(crate) struct EndpointAttr {
                method: HttpMethod,
                uri: syn::LitStr,
            }

            impl EndpointAttr {
                pub(crate) fn uri(&self) -> &syn::LitStr {
                    &self.uri
                }

                pub(crate) fn method(&self) -> HttpMethod {
                    self.method
                }

                pub(crate) fn from_attrs(attrs: &[syn::Attribute]) -> Option<Result<Self, syn::Error>> {
                    attrs.iter().find_map(|attr| {
                        $(
                            if attr.path().is_ident(stringify!($method_name)) {
                                match attr.parse_args::<syn::LitStr>() {
                                    Err(err) => return Some(Err(err)),
                                    Ok(uri) => return Some(Ok(Self {
                                        method: HttpMethod::[<$method_name:camel>],
                                        uri
                                    })),
                                }
                            }
                        ) else* else {
                            return None;
                        }
                    })
                }
            }
        }
    };
}

generate_endpoint_attr_kind!(get, post, put, patch, delete, head);
