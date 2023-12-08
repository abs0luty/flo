//! Defines a helper fn to determine if a method is async or not.

use syn::{ReturnType, Signature, Type, TypeParamBound};

/// Returns `true` if the method is async (in most cases).
pub(crate) fn is_async(method_signature: &Signature) -> bool {
    if !method_signature.asyncness.is_some() {
        if let ReturnType::Type(_, output) = &method_signature.output {
            if let Type::ImplTrait(future_trait) = output.as_ref() {
                if future_trait.bounds.iter().any(|bound| {
                    matches!(bound, TypeParamBound::Trait(trait_bound)
                    if trait_bound.path.segments.last().unwrap().ident == "Future")
                }) {
                    return true;
                }
            }
        }
    } else {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use syn::{parse_quote, ItemFn};

    use super::*;

    #[test]
    fn with_async_keyword() {
        let method: ItemFn = parse_quote! {
            async fn foo() {}
        };

        assert!(is_async(&method.sig));
    }

    #[test]
    fn returns_future() {
        let method: ItemFn = parse_quote! {
            fn foo() -> impl Future<Output = ()> {}
        };

        assert!(is_async(&method.sig));
    }

    #[test]
    fn returns_future2() {
        let method: ItemFn = parse_quote! {
            fn foo() -> impl std::future::Future<Output = ()> {}
        };

        assert!(is_async(&method.sig));
    }

    #[test]
    fn does_not_return_future() {
        let method: ItemFn = parse_quote! {
            fn foo() -> i32 {}
        };

        assert!(!is_async(&method.sig));
    }
}
