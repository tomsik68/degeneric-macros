use crate::args::{parse_degeneric_args, DegenericArg, TraitDecl};
use crate::type_tools::*;
use quote::format_ident;
use quote::quote;
use quote::quote_spanned;
use syn::punctuated::Punctuated;
use syn::Error;
use syn::Field;
use syn::GenericParam;
use syn::Generics;
use syn::Ident;
use syn::ItemStruct;
use syn::Token;
use syn::Type;
use syn::TypeParam;
use syn::TypeParamBound;
use syn::WherePredicate;

fn ident_from_ty(ty: &Type) -> Option<&Ident> {
    let path_ty = match ty {
        Type::Path(path) => path,
        _ => {
            return None;
        }
    };

    path_ty.path.get_ident()
}

fn filter_fields<'a>(fields: impl IntoIterator<Item = &'a Field>) -> Result<Vec<&'a Field>, Error> {
    let mut result = vec![];
    'outer: for f in fields {
        let args = parse_degeneric_args(&f.attrs)?;
        for arg in args {
            if let DegenericArg::NoGetter = arg {
                continue 'outer;
            }
        }
        result.push(f);
    }

    Ok(result)
}

fn extract_trait_decl(args: &[DegenericArg]) -> Result<&TraitDecl, Error> {
    args.iter()
        .flat_map(|arg| match arg {
            DegenericArg::TraitDecl(td) => Some(td),
            _ => None,
        })
        .next()
        .ok_or_else(|| {
            Error::new(
                proc_macro2::Span::call_site(),
                "#[degeneric(trait = \"pub trait Something\")] attribute is required",
            )
        })
}

fn discover_type_param_bounds<'c, 'a: 'c, 'b: 'c>(
    tp: &'a TypeParam,
    generics: &'b Generics,
) -> impl Iterator<Item = &'c TypeParamBound> {
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
}

fn determine_generic_bounds(
    params: &[&GenericParam],
    generics: &Generics,
    generic_idents: &[&Ident],
) -> Vec<Punctuated<TypeParamBound, Token![+]>> {
    params
        .iter()
        .flat_map(|gp| match gp {
            GenericParam::Type(tp) => Some(tp),
            _ => None,
        })
        .map(|tp| discover_type_param_bounds(tp, generics))
        .map(|bounds| {
            bounds
                .cloned()
                .map(|b| bound_to_associated_ty(b, generic_idents))
                .collect()
        })
        .collect()
}

fn determine_trait_generics(generics: &Generics) -> Result<Vec<&GenericParam>, Error> {
    let mut result = vec![];
    for param in &generics.params {
        match param {
            x @ GenericParam::Type(tp) => {
                let attrs = parse_degeneric_args(&tp.attrs)?;
                let preserved = attrs
                    .iter()
                    .any(|x| matches!(x, DegenericArg::PreserveGeneric));
                if preserved {
                    result.push(x);
                }
            }

            x => result.push(x),
        }
    }
    Ok(result)
}

fn determine_hidden_generics(generics: &Generics) -> Result<Vec<&GenericParam>, Error> {
    let mut result = vec![];
    for param in &generics.params {
        match param {
            x @ GenericParam::Type(tp) => {
                let attrs = parse_degeneric_args(&tp.attrs)?;
                let preserved = attrs
                    .iter()
                    .any(|x| matches!(x, DegenericArg::PreserveGeneric));
                if !preserved {
                    result.push(x);
                }
            }

            x => result.push(x),
        }
    }
    Ok(result)
}

pub fn process_struct(strct: &ItemStruct) -> Result<proc_macro2::TokenStream, Error> {
    let args = parse_degeneric_args(&strct.attrs)?;
    let trait_decl = extract_trait_decl(&args)?;

    let generics = &strct.generics;
    let filtered_fields = filter_fields(strct.fields.iter())?;

    let trait_generics = determine_trait_generics(&strct.generics)?;

    let hidden_generics = determine_hidden_generics(&strct.generics)?;
    let generic_idents: Vec<_> = hidden_generics
        .iter()
        .flat_map(|gp| match gp {
            GenericParam::Type(tp) => Some(&tp.ident),
            _ => None,
        })
        .collect();
    let generic_bounds =
        determine_generic_bounds(&hidden_generics, &strct.generics, &generic_idents);

    let field_idents: Vec<_> = filtered_fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect();
    let mut_field_idents: Vec<_> = filtered_fields
        .iter()
        .filter(|field| can_be_made_mutable(&field.ty))
        .map(|field| field.ident.as_ref().unwrap())
        .collect();
    let getter_idents_mut: Vec<_> = filtered_fields
        .iter()
        .filter(|field| can_be_made_mutable(&field.ty))
        .map(|field| format_ident!("{}_mut", field.ident.as_ref().unwrap()))
        .collect();
    let field_types: Vec<_> = filtered_fields.iter().map(|field| &field.ty).collect();

    let associated_field_types: Vec<_> = field_types
        .iter()
        .map(|ty| to_associated_ty((*ty).clone(), &generic_idents))
        .map(|ty| make_reference(ty, None))
        .collect::<Result<_, _>>()?;

    let associated_field_types_mut: Vec<_> = field_types
        .iter()
        .filter(|ty| can_be_made_mutable(ty))
        .map(|ty| to_associated_ty((*ty).clone(), &generic_idents))
        .map(|ty| make_reference(ty, Some(Default::default())))
        .collect::<Result<_, _>>()?;

    let name = &strct.ident;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let trait_vis = &trait_decl.vis;
    let trait_name = &trait_decl.ident;

    let getter_decls: Vec<_> = field_idents
        .iter()
        .zip(associated_field_types.iter())
        .map(|(fi, aft)| {
            quote_spanned! { fi.span() =>
                fn #fi(&self) -> #aft;
            }
        })
        .collect();

    let getter_impls: Vec<_> = field_idents
        .iter()
        .zip(associated_field_types.iter())
        .map(|(fi, aft)| {
            quote_spanned! { fi.span() =>
                fn #fi(&self) -> #aft {
                    &self.#fi
                }
            }
        })
        .collect();

    let mut_getter_decls: Vec<_> = getter_idents_mut
        .iter()
        .zip(associated_field_types_mut.iter())
        .map(|(fi, aft)| {
            quote_spanned! { fi.span() =>
                fn #fi(&mut self) -> #aft;
            }
        })
        .collect();

    let mut_getter_impls: Vec<_> = getter_idents_mut
        .iter()
        .zip(associated_field_types_mut.iter())
        .zip(mut_field_idents.iter())
        .map(|((fi, aft), mfi)| {
            quote_spanned! { fi.span() =>
                fn #fi(&mut self) -> #aft {
                    &mut self.#mfi
                }
            }
        })
        .collect();

    let r = quote! {
        #trait_vis trait #trait_name<#(#trait_generics),*> {
            #(
            type #generic_idents: #generic_bounds;
            )*

            #(
            #getter_decls
            )*

            #(
            #mut_getter_decls
            )*

        }

        #[automatically_derived]
        impl #impl_generics #trait_name<#(#trait_generics),*> for #name #ty_generics #where_clause {
            #(
            type #generic_idents = #generic_idents;
            )*

            #(
            #getter_impls
            )*

            #(
            #mut_getter_impls
            )*
        }
    };

    Ok(r)
}
