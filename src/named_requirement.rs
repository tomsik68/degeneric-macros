use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseBuffer;
use syn::punctuated::Punctuated;
use syn::GenericParam;
use syn::Generics;
use syn::Ident;
use syn::Result;
use syn::Token;
use syn::TypeParam;
use syn::TypeParamBound;

pub struct NrInput {
    pub_token: Option<Token![pub]>,
    ident: Ident,
    generics: Generics,
    colon: Token![:],
    bounds: Punctuated<TypeParamBound, Token![+]>,
}

impl Parse for NrInput {
    fn parse(pb: &ParseBuffer) -> Result<Self> {
        let pub_token = pb.parse()?;
        let ident = pb.parse()?;
        let generics = pb.parse()?;
        let colon = pb.parse()?;
        let mut bounds = Punctuated::new();
        let first_bound = pb.parse()?;
        bounds.push(first_bound);
        while pb.peek(Token![+]) {
            let _punct: Token![+] = pb.parse()?;
            let bound = pb.parse()?;
            bounds.push(bound);
        }

        Ok(Self {
            pub_token,
            ident,
            generics,
            colon,
            bounds,
        })
    }
}

pub fn process_input(input: NrInput) -> TokenStream {
    let pub_token = input.pub_token;
    let ident = input.ident;
    let bounds = input.bounds.clone();
    let input_generics = input.generics.clone();
    let generics = {
        let mut g = input.generics;
        g.params.push(GenericParam::Type(TypeParam {
            attrs: vec![],
            ident: format_ident!("NamedRequirementName"),
            colon_token: Default::default(),
            bounds: bounds.clone(),
            default: None,
            eq_token: None,
        }));
        g
    };

    let generic_idents: Vec<_> = input_generics.type_params().map(|tp| &tp.ident).collect();
    let lifetimes: Vec<_> = input_generics.lifetimes().map(|lt| &lt.lifetime).collect();
    let lifetimes_to_types = if !lifetimes.is_empty() && !generic_idents.is_empty() {
        quote! {,}
    } else {
        quote! {}
    };

    let colon = input.colon;

    quote! {
        #pub_token trait #ident #input_generics #colon #bounds {}
        impl #generics #ident <#(#lifetimes),* #lifetimes_to_types #(#generic_idents),*> for NamedRequirementName {}
    }
}
