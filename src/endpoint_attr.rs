use paste::paste;

pub(crate) enum EndpointAttr {
    Get(syn::LitStr),
    Post(syn::LitStr),
    Put(syn::LitStr),
    Patch(syn::LitStr),
    Delete(syn::LitStr),
    Head(syn::LitStr),
}

macro_rules! generate_methods {
    ($($method_name:ident),*) => {
        paste! {
            pub(crate) fn uri(&self) -> &syn::LitStr {
                match self {
                    $(EndpointAttr::[<$method_name:camel>](uri) => uri),*
                }
            }

            pub(crate) fn snake_case_ident(&self) -> proc_macro2::Ident {
                proc_macro2::Ident::new(
                    &self.method_name(),
                    proc_macro2::Span::call_site()
                )
            }

            /// Returns method name in snake case.
            pub(crate) fn method_name(&self) -> &'static str {
                match self {
                    $(EndpointAttr::[<$method_name:camel>](_) => stringify!($method_name)),*
                }
            }

            pub(crate) fn from_attrs(attrs: &[syn::Attribute]) -> Option<Result<Self, syn::Error>> {
                attrs.iter().find_map(|attr| {
                    $(
                        if attr.path().is_ident(stringify!($method_name)) {
                            match attr.parse_args::<syn::LitStr>() {
                                Err(err) => return Some(Err(err)),
                                Ok(uri) => return Some(Ok(Self::[<$method_name:camel>](uri))),
                            }
                        }
                    ) else* else {
                        return None;
                    }
                })
            }
        }
    };
}

impl EndpointAttr {
    generate_methods!(get, post, put, patch, delete, head);
}
