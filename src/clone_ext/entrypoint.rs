use darling::{FromDeriveInput, FromField, FromMeta, ToTokens};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Generics, Ident, Path};

pub fn process_struct(input: &DeriveInput) -> syn::Result<TokenStream> {
    let ce = CloneExt::from_derive_input(input).map_err(|de| syn::Error::new(input.span(), de))?;
    Ok(quote! {
        #ce
    })
}

#[derive(Default, FromMeta, Clone)]
enum CloneBehavior {
    #[default]
    CloneValue,
    CallFunction(Path),
}

#[derive(FromField)]
#[darling(attributes(clone_ext))]
struct FieldToClone {
    ident: Option<Ident>,

    #[darling(default)]
    clone_behavior: CloneBehavior,
}

#[derive(FromDeriveInput)]
#[darling(attributes(clone_ext))]
struct CloneExt {
    ident: Ident,
    generics: Generics,

    data: darling::ast::Data<darling::util::Ignored, FieldToClone>,
}

impl ToTokens for FieldToClone {
    fn to_tokens(&self, ts: &mut TokenStream) {
        let ident = &self.ident;

        let qt = match &self.clone_behavior {
            CloneBehavior::CloneValue => quote! {
                #ident: self.#ident.clone()
            },
            CloneBehavior::CallFunction(path) => quote! {
                #ident: #path()
            },
        };
        ts.extend(qt);
    }
}

impl ToTokens for CloneExt {
    fn to_tokens(&self, ts: &mut TokenStream) {
        let ident = &self.ident;
        let (impl_generics, tys, where_clause) = self.generics.split_for_impl();
        let fields = match &self.data {
            darling::ast::Data::Struct(f) => f,
            _ => abort!(self, "CloneExt only works on structs"),
        }
        .iter();

        ts.extend(quote! {
            #[automatically_derived]
            impl #impl_generics Clone for #ident #tys #where_clause {
                fn clone(&self) -> Self {
                    Self {
                        #(
                        #fields
                        ),*
                    }
                }
            }
        });
    }
}
