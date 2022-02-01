use crate::type_tools::bound_to_associated_ty;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{GenericParam, Generics, Ident, Type, TypeParam, WherePredicate};

#[derive(Debug)]
pub struct AssociatedType(pub TypeParam);

impl From<(&TypeParam, &Generics, &Vec<&Ident>)> for AssociatedType {
    fn from((tp, generics, associated): (&TypeParam, &Generics, &Vec<&Ident>)) -> Self {
        let mut cl = tp.clone();
        cl.bounds = cl
            .bounds
            .into_iter()
            .chain(
                // add bounds on the type from generics where clause
                generics
                    .where_clause
                    .iter()
                    .flat_map(|wh| &wh.predicates)
                    .flat_map(|pr| {
                        if let WherePredicate::Type(pt) = pr {
                            Some(pt)
                        } else {
                            None
                        }
                    })
                    .flat_map(|pr| match &pr.bounded_ty {
                        Type::Path(pt) if pt.path.is_ident(&tp.ident) => {
                            Some(pr.bounds.iter().cloned())
                        }
                        _ => None,
                    })
                    .flatten()
                    .collect::<Vec<_>>(),
            )
            .map(|bound| bound_to_associated_ty(bound, associated))
            .collect();
        // ensure colon exists
        cl.colon_token = match cl.bounds.len() {
            0 => None,
            _ => Some(Default::default()),
        };
        Self(cl)
    }
}

impl ToTokens for AssociatedType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = &self.0.attrs;
        let ident = &self.0.ident;
        let colon = &self.0.colon_token;
        let bounds = &self.0.bounds;
        tokens.extend(quote! {
            #(#attrs)*
            type #ident #colon #bounds;
        });
    }
}

pub struct TraitGenerics(pub Generics);

impl From<&Generics> for TraitGenerics {
    fn from(g: &Generics) -> Self {
        let params = g
            .params
            .iter()
            .flat_map(|param| match param {
                GenericParam::Lifetime(lifetime) => Some(lifetime),
                _ => None,
            })
            .cloned()
            .map(|mut lifetime| {
                lifetime.bounds.extend(
                    g.where_clause
                        .iter()
                        .flat_map(|wh| &wh.predicates)
                        .flat_map(|pred| match pred {
                            WherePredicate::Lifetime(pl) => Some(pl),
                            _ => None,
                        })
                        .filter(|pl| pl.lifetime == lifetime.lifetime)
                        .flat_map(|pl| pl.bounds.iter())
                        .cloned()
                        .collect::<Vec<_>>(),
                );
                lifetime
            })
            .map(GenericParam::Lifetime)
            .collect();
        Self(Generics {
            lt_token: g.lt_token,
            gt_token: g.gt_token,
            params,
            where_clause: None,
        })
    }
}

impl ToTokens for TraitGenerics {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}
