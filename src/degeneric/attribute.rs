use darling::{FromAttributes, FromMeta, Result};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseBuffer},
    Attribute, Ident, Token, Visibility,
};

pub struct TraitDecl {
    pub vis: Visibility,
    pub trait_kw: Token![trait],
    pub ident: Ident,
}

impl ToTokens for TraitDecl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vis = &self.vis;
        let trait_kw = &self.trait_kw;
        let ident = &self.ident;
        tokens.extend(quote! {
            #vis #trait_kw #ident
        });
    }
}

impl Parse for TraitDecl {
    fn parse(pb: &ParseBuffer) -> syn::Result<Self> {
        Ok(Self {
            vis: pb.parse()?,
            trait_kw: pb.parse()?,
            ident: pb.parse()?,
        })
    }
}

impl FromMeta for TraitDecl {
    fn from_string(value: &str) -> Result<Self> {
        Ok(syn::parse_str(value)?)
    }
}

#[derive(FromAttributes)]
#[darling(attributes(degeneric))]
pub struct DegenericTypeAttrs {
    #[allow(unused)]
    #[darling(default)]
    pub preserve: Option<()>,
}

pub struct Attrs(Vec<Attribute>);

impl Parse for Attrs {
    fn parse(pb: &ParseBuffer) -> syn::Result<Self> {
        Ok(Self(Attribute::parse_outer(pb)?))
    }
}

impl FromMeta for Attrs {
    fn from_string(value: &str) -> Result<Self> {
        Ok(syn::parse_str(value)?)
    }
}

impl ToTokens for Attrs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in &self.0 {
            attr.to_tokens(tokens);
        }
    }
}
