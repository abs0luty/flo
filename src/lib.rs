use endpoint_attr::EndpointAttr;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_quote};

mod endpoint_attr;

/// ```
/// #[api(TestApiClient)]
/// trait TestApi {
///     #[get("/tasks")]
///     async fn get_tasks(
///         &self,
///         #[query] page: Option<i32>,
///     ) -> Result<Vec<Task>, Error>;
///
///     #[post("/tasks")]
///     async fn create_task(
///         &self,
///         #[body] task: Task,
///     ) -> Result<(), Error>;
///
///     #[put("/tasks/{}")]
///     async fn update_task(
///         &self,
///         #[path] task_id: i32,
///         #[body] task: Task,
///     ) -> Result<(), Error>;
///
///     #[delete("/tasks/{}")]
///     async fn delete_task(
///         &self,
///         #[path] task_id: i32,
///     ) -> Result<(), Error>;
/// }
/// ```
#[proc_macro_attribute]
pub fn api(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
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

struct ApiMacroInput {
    api_trait_name: syn::Ident,
    methods: Vec<(syn::TraitItemFn, EndpointAttr)>,
}

fn parse_api_macro_input(item_trait: syn::ItemTrait) -> Result<ApiMacroInput, syn::Error> {
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

        if !is_async(method) {
            return Err(syn::Error::new_spanned(method, "method must be async"));
        }

        methods.push((method.clone(), endpoint_attr));
    }

    Ok(ApiMacroInput {
        api_trait_name,
        methods,
    })
}

fn is_async(method: &syn::TraitItemFn) -> bool {
    if !method.sig.asyncness.is_some() {
        if let syn::ReturnType::Type(_, output) = &method.sig.output {
            if let syn::Type::ImplTrait(future_trait) = output.as_ref() {
                if future_trait
                    .bounds
                    .iter()
                    .any(|bound| matches!(bound, syn::TypeParamBound::Trait(trait_bound) if trait_bound.path.is_ident("Future")))
                {
                    return true;
                }
            }
        }
    } else {
        return true;
    }

    false
}

fn expand_api_macro(
    api_client_struct_name: syn::Ident,
    api_trait_name: syn::Ident,
    methods: Vec<(syn::TraitItemFn, EndpointAttr)>,
    item_trait: syn::ItemTrait,
) -> TokenStream {
    let mut expanded = TokenStream::new();

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
    api_client_struct_name: &syn::Ident,
    api_trait_name: &syn::Ident,
    methods: impl IntoIterator<Item = (syn::TraitItemFn, EndpointAttr)>,
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

fn generate_api_client_struct(api_client_struct_name: &syn::Ident) -> TokenStream {
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

            pub fn with_basic_auth(mut self, username: impl Display, password: impl Display) -> Self {
                self.default_headers
                    .insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(
                        &format!("Basic {}", base64::encode(format!("{username}:{password}"))))
                        .unwrap());
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
            pub const fn base_url(&self) -> &str {
                &self.base_url
            }
        }
    }
}

fn generate_api_client_method_impl(
    method: syn::TraitItemFn,
    endpoint_attr: EndpointAttr,
) -> TokenStream {
    let http_method_name = endpoint_attr.snake_case_ident();
    let method_name = &method.sig.ident;
    let uri = endpoint_attr.uri();

    let mut generated_method: syn::ItemFn = parse_quote! {
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
    generated_method.into_token_stream()
}
