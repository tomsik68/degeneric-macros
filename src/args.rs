use std::convert::TryFrom;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Ident, Lit, Meta, NestedMeta, Token, Visibility};

pub struct TraitDecl {
    pub vis: Visibility,
    pub trait_kw: Token![trait],
    pub ident: Ident,
}

impl TryFrom<Lit> for TraitDecl {
    type Error = Error;

    fn try_from(lit: Lit) -> Result<Self, Self::Error> {
        match lit {
            Lit::Str(ref s) => Ok(s.parse().map_err(|e| Error::new(lit.span(), e))?),
            _ => Err(Error::new(
                lit.span(),
                "unknown literal type in trait declaration. should be #[degeneric(trait = \"pub trait Something\")]",
            )),
        }
    }
}

impl Parse for TraitDecl {
    fn parse(pb: &syn::parse::ParseBuffer<'_>) -> Result<Self, syn::Error> {
        Ok(Self {
            vis: pb.parse()?,
            trait_kw: pb.parse()?,
            ident: pb.parse()?,
        })
    }
}

pub enum DegenericArg {
    TraitDecl(TraitDecl),
    NoGetter,
    PreserveGeneric,
}

impl TryFrom<NestedMeta> for DegenericArg {
    type Error = Error;

    fn try_from(meta: NestedMeta) -> Result<Self, Self::Error> {
        match meta {
            NestedMeta::Meta(meta) => match meta {
                Meta::NameValue(nv) => {
                    let key = nv
                        .path
                        .get_ident()
                        .ok_or_else(|| {
                            Error::new(nv.path.span(), "attribute key must be a single identifier")
                        })?
                        .to_string();
                    match key.as_ref() {
                        "trait" => Ok(Self::TraitDecl(TraitDecl::try_from(nv.lit)?)),
                        _ => Err(Error::new(nv.path.span(), "unrecognized property")),
                    }
                }
                Meta::Path(pt) => {
                    let key = pt
                        .get_ident()
                        .ok_or_else(|| {
                            Error::new(pt.span(), "attribute key must be a single identifier")
                        })?
                        .to_string();
                    match key.as_ref() {
                        "no_getter" => Ok(Self::NoGetter),
                        "preserve" => Ok(Self::PreserveGeneric),
                        _ => Err(Error::new(pt.span(), "unknown attribute")),
                    }
                }
                mt => Err(Error::new(
                    mt.span(),
                    "unable to parse degeneric attributes",
                )),
            },
            mt => Err(Error::new(
                mt.span(),
                "unable to parse degeneric attributes",
            )),
        }
    }
}

pub fn parse_degeneric_args(attrs: &[Attribute]) -> Result<Vec<DegenericArg>, Error> {
    let mut result = vec![];

    for attr in attrs {
        if !attr.path.is_ident("degeneric") {
            continue;
        }

        let meta = attr.parse_meta()?;
        let meta = match meta {
            Meta::List(ls) => Ok(ls.nested.into_iter()),
            x => Err(Error::new(
                x.span(),
                "unknown attributes for degeneric attribute",
            )),
        }?;

        for nm in meta {
            result.push(DegenericArg::try_from(nm)?);
        }
    }

    Ok(result)
}
