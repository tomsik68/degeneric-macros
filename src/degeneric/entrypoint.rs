use super::type_tools::can_be_made_mutable;
use darling::FromAttributes;
use darling::{FromDeriveInput, Result, ToTokens};
use proc_macro2::TokenStream;

use quote::quote;
use syn::spanned::Spanned;

use syn::{Attribute, DeriveInput, Generics, Ident};

use super::attribute::*;
use super::field::*;
use super::generics::*;

#[derive(FromDeriveInput)]
#[darling(attributes(degeneric), supports(struct_named))]
#[darling(attributes(degeneric), forward_attrs(allow, cfg, cfg_attr, doc))]
struct Degeneric {
    ident: Ident,
    generics: Generics,

    #[darling(rename = "trait")]
    trait_decl: TraitDecl,

    attrs: Vec<Attribute>,

    #[darling(multiple)]
    trait_decl_attr: Vec<Attrs>,

    #[darling(multiple)]
    trait_impl_attr: Vec<Attrs>,

    #[darling(default)]
    dynamize: Option<()>,

    data: darling::ast::Data<darling::util::Ignored, FieldDecl>,
}

impl ToTokens for Degeneric {
    fn to_tokens(&self, ts: &mut TokenStream) {
        let decl = &self.trait_decl;
        let trait_decl_attr = &self.trait_decl_attr;
        let trait_impl_attr = &self.trait_impl_attr;
        let attrs = &self.attrs;
        let generics = &self.generics;
        let trait_generics = TraitGenerics::from(generics);
        let trait_name = &self.trait_decl.ident;
        let ident = &self.ident;
        let (impl_generics, tys, where_clause) = generics.split_for_impl();
        let (_, trait_ty_generics, _) = trait_generics.0.split_for_impl();

        let associated_types_idents: Result<Vec<_>> = self
            .generics
            .type_params()
            .map(|tp| Ok((tp, DegenericTypeAttrs::from_attributes(&tp.attrs)?)))
            .filter(|res| match res {
                Ok((_, attrs)) => attrs.preserve.is_none(),
                _ => true,
            })
            .collect();

        let associated_types_idents = pme_unwrap!(
            associated_types_idents,
            generics.span(),
            "failed to get associated types idents: {err}"
        );

        let associated_types_idents: Vec<_> = associated_types_idents
            .into_iter()
            .map(|(tp, _)| &tp.ident)
            .collect();

        let associated_types: Vec<_> = self
            .generics
            .type_params()
            .map(|tp| AssociatedType::from((tp, generics, &associated_types_idents)))
            .collect();

        let dynamize = if self.dynamize.is_some() {
            super::dynamize::emit_dynamize(&associated_types).into_token_stream()
        } else {
            quote! {}.into_token_stream()
        };

        let associated_types_impl: Vec<_> = associated_types
            .iter()
            .map(|at| {
                let name = &at.0.ident;
                quote! {
                    type #name = #name;
                }
            })
            .collect();

        let associated_types_idents: Vec<_> =
            associated_types.iter().map(|ty| &ty.0.ident).collect();

        let getter_decls: Vec<_> = self
            .data
            .as_ref()
            .take_struct()
            .as_ref()
            .unwrap()
            .iter()
            .filter(|f| f.no_getter.is_none())
            .map(|f| f.declare_getter(&associated_types_idents))
            .collect();

        let mut_getter_decls: Vec<_> = self
            .data
            .as_ref()
            .take_struct()
            .as_ref()
            .unwrap()
            .iter()
            .filter(|f| can_be_made_mutable(&f.ty))
            .filter(|f| f.no_getter.is_none())
            .map(|f| f.declare_mut_getter(&associated_types_idents))
            .collect();

        let getter_impls: Vec<_> = self
            .data
            .as_ref()
            .take_struct()
            .as_ref()
            .unwrap()
            .iter()
            .filter(|f| f.no_getter.is_none())
            .map(|f| f.implement_getter(&associated_types_idents))
            .collect();

        let mut_getter_impls: Vec<_> = self
            .data
            .as_ref()
            .take_struct()
            .as_ref()
            .unwrap()
            .iter()
            .filter(|f| can_be_made_mutable(&f.ty))
            .filter(|f| f.no_getter.is_none())
            .map(|f| f.implement_mut_getter(&associated_types_idents))
            .collect();

        ts.extend(quote! {

            #(#attrs)*
            #(#trait_decl_attr)*
            #dynamize
            #decl #trait_generics {
                #(#associated_types)*

                #(#getter_decls)*

                #(#mut_getter_decls)*
            }

            #(#attrs)*
            #(#trait_impl_attr)*
            #[automatically_derived]
            impl #impl_generics #trait_name #trait_ty_generics for #ident #tys #where_clause {

                #(#associated_types_impl)*

                #(#getter_impls)*
                #(#mut_getter_impls)*
            }

        });
    }
}

pub fn process_struct(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let dg = Degeneric::from_derive_input(input).map_err(|de| syn::Error::new(input.span(), de))?;
    Ok(quote! {
        #dg
    })
}
