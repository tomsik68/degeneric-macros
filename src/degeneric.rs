use quote::format_ident;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::GenericParam;
use syn::Generics;
use syn::Ident;
use syn::ItemStruct;
use syn::Token;
use syn::Type;
use syn::WherePredicate;

use crate::type_tools::*;

fn ident_from_ty(ty: &Type) -> Option<&Ident> {
    let path_ty = match ty {
        Type::Path(path) => path,
        _ => {
            return None;
        }
    };

    let mut segments = path_ty.path.segments.iter();
    let one = segments.next().map(|ps| &ps.ident);
    let last = segments.last();

    match last {
        Some(_) => None,
        None => one,
    }
}

pub fn process_struct(strct: &ItemStruct) -> proc_macro2::TokenStream {
    let trait_name = format_ident!("{}Trait", strct.ident);
    let generics = &strct.generics;

    let lifetimes: Vec<_> = strct.generics.lifetimes().map(|lt| &lt.lifetime).collect();
    let lifetime_colons: Vec<_> = strct
        .generics
        .lifetimes()
        .map(|lt| lt.colon_token)
        .collect();
    let lifetime_bounds: Vec<_> = strct.generics.lifetimes().map(|lt| &lt.bounds).collect();

    let generic_idents: Vec<_> = strct.generics.type_params().map(|tp| &tp.ident).collect();
    let generic_bounds: Vec<_> = strct
        .generics
        .type_params()
        .map(|tp| {
            let tp_ident = tp.ident.clone();
            tp.bounds.iter().chain(
                generics
                    .where_clause
                    .iter()
                    .flat_map(|wh| wh.predicates.iter())
                    .filter_map(move |pred| match pred {
                        WherePredicate::Type(pt) => {
                            if ident_from_ty(&pt.bounded_ty) == Some(&tp_ident) {
                                Some(pt.bounds.iter())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .flatten(),
            )
        })
        .map(|bounds| {
            bounds
                .cloned()
                .map(|b| bound_to_associated_ty(b, &generic_idents))
                .collect::<Punctuated<_, Token![+]>>()
        })
        .collect();

    let field_idents: Vec<_> = strct
        .fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect();
    let field_idents_mut: Vec<_> = field_idents
        .iter()
        .map(|id| format_ident!("{}_mut", id))
        .collect();
    let field_types: Vec<_> = strct.fields.iter().map(|field| &field.ty).collect();
    let associated_field_types: Vec<_> = field_types
        .iter()
        .map(|ty| to_associated_ty((*ty).clone(), &generic_idents))
        .collect();
    let name = &strct.ident;
    let impl_generics = Generics {
        gt_token: generics.gt_token,
        lt_token: generics.lt_token,
        where_clause: generics.where_clause.clone(),
        params: generics
            .params
            .iter()
            .map(|gp| match gp {
                GenericParam::Type(tp) => {
                    let mut tp = tp.clone();
                    tp.default = None;
                    GenericParam::Type(tp)
                }
                x => x.clone(),
            })
            .collect(),
    };

    let impl_where = impl_generics.where_clause.clone();

    let lifetime_to_type_params = if !lifetimes.is_empty() && !generic_idents.is_empty() {
        quote! {,}
    } else {
        quote! {}
    };

    let r = quote! {
        pub trait #trait_name<#(#lifetimes #lifetime_colons #lifetime_bounds),*> {
            #(
            type #generic_idents: #generic_bounds;
            )*

            #(
            fn #field_idents(&self) -> &#associated_field_types;
            fn #field_idents_mut(&mut self) -> &mut #associated_field_types;
            )*
        }

        impl #impl_generics #trait_name<#(#lifetimes),*> for #name <#(#lifetimes),* #lifetime_to_type_params #(#generic_idents),*> #impl_where {
            #(
            type #generic_idents = #generic_idents;
            )*
            #(
            fn #field_idents(&self) -> &#associated_field_types {
                &self.#field_idents
            }
            fn #field_idents_mut(&mut self) -> &mut #associated_field_types {
                &mut self.#field_idents
            }
            )*
        }
    };
    r
}
