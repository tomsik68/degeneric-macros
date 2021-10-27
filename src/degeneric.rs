use quote::format_ident;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::GenericParam;
use syn::Generics;
use syn::ItemStruct;
use syn::Token;

use crate::type_tools::*;

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
        .map(|tp| &tp.bounds)
        .map(|bounds| {
            bounds
                .iter()
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
    let setters: Vec<_> = field_idents
        .iter()
        .map(|id| format_ident!("with_{}", id))
        .collect();
    let initializers_without: Vec<_> = field_idents
        .iter()
        .map(|id| {
            let to_init = field_idents.iter().filter(|id2| &id != id2);
            quote! {
                #(
                    #to_init: self.#to_init
                ),*
            }
        })
        .collect();
    let name = &strct.ident;
    let builder = format_ident!("{}Builder", name);
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

    let lifetime_to_type_params = if !lifetimes.is_empty() && !generic_idents.is_empty() {
        quote! {,}
    } else {
        quote! {}
    };

    let r = quote! {
        impl #impl_generics #name <#(#lifetimes),* #lifetime_to_type_params #(#generic_idents),*> {
            pub fn builder() -> #builder <#(#lifetimes),* #(#generic_idents),*> {
                Default::default()
            }
        }


        pub trait #trait_name<#(#lifetimes #lifetime_colons #lifetime_bounds),*> {
            #(
            type #generic_idents: #generic_bounds;
            )*

            #(
            fn #field_idents(&self) -> &#associated_field_types;
            fn #field_idents_mut(&mut self) -> &mut #associated_field_types;
            )*
        }

        impl #impl_generics #trait_name<#(#lifetimes),*> for #name <#(#lifetimes),* #lifetime_to_type_params #(#generic_idents),*> {
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

        pub struct #builder #generics {
            #(
            #field_idents: Option<#field_types>
            ),*
        }

        impl #impl_generics Default for #builder <#(#lifetimes),*#(#generic_idents),*> {
            fn default() -> Self {
                Self {
                    #(
                    #field_idents: None
                    ),*
                }
            }
        }

        impl #impl_generics #builder <#(#lifetimes),* #lifetime_to_type_params #(#generic_idents),*> {
            #(
            pub fn #setters(self, val: #field_types) -> Self {
                Self {
                    #field_idents: Some(val),
                    #initializers_without
                }
            }
            )*

            pub fn build(self) -> #name <#(#lifetimes),* #lifetime_to_type_params #(#generic_idents),*> {
                #name {
                    #(
                    #field_idents: self.#field_idents.unwrap_or_else(|| panic!("degeneric: while building a {}, {} is required", stringify!(#name), stringify!(#field_idents)))
                    ),*
                }
            }
        }
    };
    r
}
