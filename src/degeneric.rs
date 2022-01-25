use crate::args::{parse_degeneric_args, Attr, AttributeScope, DegenericArg, TraitDecl};
use crate::type_tools::*;
use quote::format_ident;
use quote::quote;
use quote::quote_spanned;
use syn::punctuated::Punctuated;
use syn::Attribute;
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

fn extract_attrs_owned(args: Vec<DegenericArg>, scope: AttributeScope) -> Vec<Attribute> {
    args.into_iter()
        .flat_map(|arg| match arg {
            DegenericArg::Attr(Attr(attr), scopes) => {
                if scopes.iter().any(|sc| sc == &scope) {
                    Some(attr)
                } else {
                    None
                }
            }
            _ => None,
        })
        .flatten()
        .collect()
}

fn extract_attrs(args: &[DegenericArg], scope: AttributeScope) -> Vec<&Attribute> {
    args.iter()
        .flat_map(|arg| match arg {
            DegenericArg::Attr(Attr(attr), scopes) => {
                if scopes.iter().any(|sc| sc == &scope) {
                    Some(attr)
                } else {
                    None
                }
            }
            _ => None,
        })
        .flatten()
        .collect()
}

fn extract_doc(args: &[Attribute]) -> Option<&Attribute> {
    args.iter().find(|attr| attr.path.is_ident("doc"))
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

    let trait_doc = extract_doc(&strct.attrs);
    let trait_attrs = extract_attrs(&args, AttributeScope::TraitDecl);
    let impl_attrs = extract_attrs(&args, AttributeScope::TraitImpl);

    let generics = &strct.generics;
    let filtered_fields = filter_fields(strct.fields.iter())?;

    let trait_generics = determine_trait_generics(&strct.generics)?;

    let hidden_generics = determine_hidden_generics(&strct.generics)?;
    let generic_types: Vec<_> = hidden_generics
        .iter()
        .flat_map(|gp| match gp {
            GenericParam::Type(tp) => Some(tp),
            _ => None,
        })
        .collect();
    let generics_attrs: Vec<&Vec<_>> = generic_types.iter().map(|ty| &ty.attrs).collect();
    let generic_idents: Vec<_> = generic_types.iter().map(|tp| &tp.ident).collect();

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

    let getter_decl_attrs: Result<Vec<_>, Error> = filtered_fields
        .iter()
        .map(|f| parse_degeneric_args(f.attrs.as_slice()))
        .map(|deg_args| deg_args.map(|x| extract_attrs_owned(x, AttributeScope::GetterDecl)))
        .collect();

    let getter_decl_attrs = getter_decl_attrs?;

    let getter_impl_attrs: Result<Vec<_>, Error> = filtered_fields
        .iter()
        .map(|f| parse_degeneric_args(f.attrs.as_slice()))
        .map(|deg_args| deg_args.map(|x| extract_attrs_owned(x, AttributeScope::GetterImpl)))
        .collect();

    let getter_impl_attrs = getter_impl_attrs?;

    let getter_docs: Vec<_> = filtered_fields
        .iter()
        .map(|f| extract_doc(f.attrs.as_slice()))
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

    let mut_getter_decl_attrs: Result<Vec<_>, Error> = filtered_fields
        .iter()
        .filter(|field| can_be_made_mutable(&field.ty))
        .map(|f| parse_degeneric_args(f.attrs.as_slice()))
        .map(|deg_args| deg_args.map(|x| extract_attrs_owned(x, AttributeScope::MutGetterDecl)))
        .collect();
    let mut_getter_decl_attrs = mut_getter_decl_attrs?;

    let mut_getter_impl_attrs: Result<Vec<_>, Error> = filtered_fields
        .iter()
        .filter(|field| can_be_made_mutable(&field.ty))
        .map(|f| parse_degeneric_args(f.attrs.as_slice()))
        .map(|deg_args| deg_args.map(|x| extract_attrs_owned(x, AttributeScope::MutGetterImpl)))
        .collect();
    let mut_getter_impl_attrs = mut_getter_impl_attrs?;

    let mut_getter_docs: Vec<_> = filtered_fields
        .iter()
        .filter(|field| can_be_made_mutable(&field.ty))
        .map(|f| extract_doc(f.attrs.as_slice()))
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
        #(
        #trait_attrs
        )*
        #trait_doc
        #trait_vis trait #trait_name<#(#trait_generics),*> {
            #(
            #(
            #generics_attrs
            )*
            type #generic_idents: #generic_bounds;
            )*

            #(
            #(
            #getter_decl_attrs
            )*
            #getter_docs
            #getter_decls
            )*

            #(
            #(
            #mut_getter_decl_attrs
            )*
            #mut_getter_docs
            #mut_getter_decls
            )*

        }

        #[automatically_derived]
        #(
        #impl_attrs
        )*
        impl #impl_generics #trait_name<#(#trait_generics),*> for #name #ty_generics #where_clause {
            #(
            type #generic_idents = #generic_idents;
            )*

            #(
            #(
            #getter_impl_attrs
            )*
            #getter_impls
            )*

            #(
            #(
            #mut_getter_impl_attrs
            )*
            #mut_getter_impls
            )*
        }
    };

    Ok(r)
}
