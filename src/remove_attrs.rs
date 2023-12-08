//! Remove attributes from generated code, after the `api` macro was
//! expanded.

use syn::{
    visit_mut::{self, VisitMut},
    Attribute, FnArg, ImplItemFn, TraitItemFn,
};

pub(crate) struct RemoveAttrsVisitMut;

fn remove_endpoint_attrs(attrs: &mut Vec<Attribute>) {
    attrs.retain(|a| {
        !a.meta.path().is_ident("get")
            && !a.meta.path().is_ident("post")
            && !a.meta.path().is_ident("delete")
            && !a.meta.path().is_ident("put")
            && !a.meta.path().is_ident("head")
            && !a.meta.path().is_ident("patch")
    });
}

fn remove_param_attrs(attrs: &mut Vec<Attribute>) {
    attrs.retain(|a| {
        !a.meta.path().is_ident("body")
            && !a.meta.path().is_ident("query_param")
            && !a.meta.path().is_ident("path_param")
            && !a.meta.path().is_ident("header")
            && !a.meta.path().is_ident("cookie")
    })
}

impl VisitMut for RemoveAttrsVisitMut {
    fn visit_trait_item_fn_mut(&mut self, i: &mut TraitItemFn) {
        remove_endpoint_attrs(&mut i.attrs);

        visit_mut::visit_trait_item_fn_mut(self, i);
    }

    fn visit_impl_item_fn_mut(&mut self, i: &mut ImplItemFn) {
        remove_endpoint_attrs(&mut i.attrs);

        visit_mut::visit_impl_item_fn_mut(self, i);
    }

    fn visit_fn_arg_mut(&mut self, i: &mut FnArg) {
        if let syn::FnArg::Typed(i) = i {
            remove_param_attrs(&mut i.attrs);
        }

        visit_mut::visit_fn_arg_mut(self, i);
    }
}
