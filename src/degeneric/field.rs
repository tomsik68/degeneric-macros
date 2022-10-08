use super::attribute::Attrs;
use super::type_tools::{make_reference, to_associated_ty};
use darling::FromField;
use quote::format_ident;
use syn::spanned::Spanned;
use syn::{Attribute, Ident, TraitItem, Type};

#[derive(FromField)]
#[darling(attributes(degeneric), forward_attrs(allow, cfg, cfg_attr, doc))]
pub struct FieldDecl {
    pub ident: Option<Ident>,
    pub ty: Type,

    pub attrs: Vec<Attribute>,

    #[darling(multiple)]
    pub getter_decl_attr: Vec<Attrs>,

    #[darling(multiple)]
    pub getter_impl_attr: Vec<Attrs>,

    #[darling(multiple)]
    pub mut_getter_decl_attr: Vec<Attrs>,

    #[darling(multiple)]
    pub mut_getter_impl_attr: Vec<Attrs>,

    #[darling(default)]
    pub no_getter: Option<()>,
}

impl FieldDecl {
    pub fn declare_getter(&self, associated_types_idents: &[&Ident]) -> TraitItem {
        let name = self.ident.as_ref().unwrap();
        let docs = &self.attrs;
        let attrs = &self.getter_decl_attr;
        let return_type = pme_unwrap!(
            make_reference(
                to_associated_ty(self.ty.clone(), associated_types_idents),
                None,
            ),
            self.ty.span(),
            "unable to turn the type into a reference: {err}"
        );

        TraitItem::Method(syn::parse_quote! {
            #( #attrs )*
            #( #docs )*
            fn #name (&self) -> #return_type;
        })
    }

    pub fn implement_getter(&self, associated_types_idents: &[&Ident]) -> TraitItem {
        let name = self.ident.as_ref().unwrap();
        let docs = &self.attrs;
        let attrs = &self.getter_impl_attr;
        let return_type = pme_unwrap!(
            make_reference(
                to_associated_ty(self.ty.clone(), associated_types_idents),
                None,
            ),
            self.ty.span(),
            "unable to turn the type into a reference: {err}"
        );

        TraitItem::Method(syn::parse_quote! {
            #( #attrs )*
            #( #docs )*
            fn #name (&self) -> #return_type {
                &self.#name
            }
        })
    }

    pub fn declare_mut_getter(&self, associated_types_idents: &[&Ident]) -> TraitItem {
        let name = format_ident!("{}_mut", self.ident.as_ref().unwrap());
        let attrs = &self.mut_getter_decl_attr;
        let docs = &self.attrs;
        let return_type = pme_unwrap!(
            make_reference(
                to_associated_ty(self.ty.clone(), associated_types_idents),
                Some(syn::parse_quote! {mut}),
            ),
            self.ty.span(),
            "unable to turn the type into a reference: {err}"
        );

        TraitItem::Method(syn::parse_quote! {
            #( #attrs )*
            #( #docs )*
            fn #name (&mut self) -> #return_type;
        })
    }

    pub fn implement_mut_getter(&self, associated_types_idents: &[&Ident]) -> TraitItem {
        let field = self.ident.as_ref().unwrap();
        let name = format_ident!("{}_mut", self.ident.as_ref().unwrap());
        let attrs = &self.mut_getter_impl_attr;
        let docs = &self.attrs;
        let return_type = pme_unwrap!(
            make_reference(
                to_associated_ty(self.ty.clone(), associated_types_idents),
                Some(syn::parse_quote! {mut}),
            ),
            self.ty.span(),
            "unable to turn the type into a reference: {err}"
        );

        TraitItem::Method(syn::parse_quote! {
            #( #attrs )*
            #( #docs )*
            fn #name (&mut self) -> #return_type {
                &mut self.#field
            }
        })
    }
}
