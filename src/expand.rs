//! Code generation logic for the `api` macro.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, visit_mut::VisitMut, Ident, ImplItemFn, ItemTrait, TraitItemFn};

use crate::{endpoint::EndpointAttr, remove_attrs::RemoveAttrsVisitMut};

pub(crate) fn expand_api_macro(
    api_client_struct_name: Ident,
    api_trait_name: Ident,
    methods: Vec<(TraitItemFn, EndpointAttr)>,
    mut item_trait: ItemTrait,
) -> TokenStream {
    let mut expanded = TokenStream::new();

    RemoveAttrsVisitMut.visit_item_trait_mut(&mut item_trait);
    expanded.extend(item_trait.into_token_stream());
    expanded.extend(generate_api_client_struct(&api_client_struct_name));
    expanded.extend(generate_api_trait_impl(
        &api_client_struct_name,
        &api_trait_name,
        methods,
    ));

    expanded
}

fn generate_api_trait_impl(
    api_client_struct_name: &Ident,
    api_trait_name: &Ident,
    methods: impl IntoIterator<Item = (TraitItemFn, EndpointAttr)>,
) -> TokenStream {
    let mut api_client_methods = TokenStream::new();
    for (method, endpoint_attr) in methods {
        api_client_methods.extend(generate_api_client_method_impl(method, endpoint_attr));
    }

    quote! {
        #[async_trait::async_trait]
        impl #api_trait_name for #api_client_struct_name {
            #api_client_methods
        }
    }
}

fn generate_api_client_struct(api_client_struct_name: &Ident) -> TokenStream {
    quote! {
        struct #api_client_struct_name {
            client: reqwest::Client,
            base_url: String,
            default_headers: reqwest::header::HeaderMap,
        }

        impl #api_client_struct_name {
            #[inline]
            #[must_use]
            pub fn new(base_url: impl Into<String>) -> Self {
                Self {
                    client: reqwest::Client::new(),
                    base_url: base_url.into(),
                    default_headers: reqwest::header::HeaderMap::new(),
                }
            }

            pub fn with_default_header(
                mut self,
                key: impl reqwest::header::IntoHeaderName,
                value: impl AsRef<str>) -> Self {
                self.default_headers
                    .insert(key, reqwest::header::HeaderValue::from_str(value.as_ref()).unwrap());
                self
            }

            pub fn with_default_headers(mut self, headers: reqwest::header::HeaderMap) -> Self {
                self.default_headers.extend(headers);
                self
            }

            pub fn with_basic_auth(mut self,
                username: impl std::fmt::Display,
                password: impl std::fmt::Display) -> Self {
                use base64::Engine;

                self.default_headers
                    .insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(
                        &format!("Basic {}", base64::engine::general_purpose::STANDARD
                            .encode(format!("{username}:{password}"))
                        )).unwrap());
                self
            }

            pub fn with_bearer_auth(mut self, token: String) -> Self {
                self.default_headers
                    .insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(
                        &format!("Bearer {}", token))
                        .unwrap());
                self
            }

            #[inline]
            #[must_use]
            pub const fn client(&self) -> &reqwest::Client {
                &self.client
            }

            #[inline]
            #[must_use]
            pub fn base_url(&self) -> &str {
                &self.base_url
            }
        }
    }
}

fn generate_api_client_method_impl(
    method: TraitItemFn,
    endpoint_attr: EndpointAttr,
) -> TokenStream {
    let http_method_name = endpoint_attr.method().snake_case_ident();
    let method_name = &method.sig.ident;
    let uri = endpoint_attr.uri();

    let mut generated_method: ImplItemFn = parse_quote! {
        async fn #method_name(&self) {
            let url = format!("{}/{}", self.base_url, #uri);

            self.client
                .#http_method_name(url)
                .headers(self.default_headers.clone())
                .send()
                .await
        }
    };

    generated_method.sig = method.sig;
    RemoveAttrsVisitMut.visit_impl_item_fn_mut(&mut generated_method);
    generated_method.into_token_stream()
}
