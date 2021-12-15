use quote::format_ident;
use syn::punctuated::Punctuated;
use syn::AngleBracketedGenericArguments;
use syn::BareFnArg;
use syn::GenericArgument;
use syn::Ident;
use syn::ParenthesizedGenericArguments;
use syn::Path;
use syn::PathArguments;
use syn::PathSegment;
use syn::ReturnType;
use syn::Type;
use syn::TypeArray;
use syn::TypeBareFn;
use syn::TypeGroup;
use syn::TypeParamBound;
use syn::TypeParen;
use syn::TypePath;
use syn::TypePtr;
use syn::TypeReference;
use syn::TypeSlice;
use syn::TypeTuple;

fn array_to_associated_ty(ta: TypeArray, generic_idents: &[&Ident]) -> TypeArray {
    TypeArray {
        bracket_token: ta.bracket_token,
        semi_token: ta.semi_token,
        len: ta.len,
        elem: Box::new(to_associated_ty(*ta.elem, generic_idents)),
    }
}

fn bare_fn_arg_to_associated_ty(bfa: BareFnArg, generic_idents: &[&Ident]) -> BareFnArg {
    BareFnArg {
        attrs: bfa.attrs,
        name: bfa.name,
        ty: to_associated_ty(bfa.ty, generic_idents),
    }
}

fn return_type_to_associated_ty(rt: ReturnType, generic_idents: &[&Ident]) -> ReturnType {
    match rt {
        ReturnType::Type(rarr, ty) => {
            ReturnType::Type(rarr, Box::new(to_associated_ty(*ty, generic_idents)))
        }
        x => x,
    }
}

fn bare_fn_to_associated_ty(bft: TypeBareFn, generic_idents: &[&Ident]) -> TypeBareFn {
    let inputs = bft
        .inputs
        .into_iter()
        .map(|inp| bare_fn_arg_to_associated_ty(inp, generic_idents))
        .collect();
    let output = return_type_to_associated_ty(bft.output, generic_idents);

    TypeBareFn {
        lifetimes: bft.lifetimes,
        abi: bft.abi,
        fn_token: bft.fn_token,
        inputs,
        output,
        paren_token: bft.paren_token,
        unsafety: bft.unsafety,
        variadic: bft.variadic,
    }
}

fn group_to_associated_ty(gr: TypeGroup, generic_idents: &[&Ident]) -> TypeGroup {
    TypeGroup {
        group_token: gr.group_token,
        elem: Box::new(to_associated_ty(*gr.elem, generic_idents)),
    }
}

fn paren_to_associated_ty(pa: TypeParen, generic_idents: &[&Ident]) -> TypeParen {
    TypeParen {
        paren_token: pa.paren_token,
        elem: Box::new(to_associated_ty(*pa.elem, generic_idents)),
    }
}

fn ptr_to_associated_ty(ptr: TypePtr, generic_idents: &[&Ident]) -> TypePtr {
    TypePtr {
        const_token: ptr.const_token,
        elem: Box::new(to_associated_ty(*ptr.elem, generic_idents)),
        mutability: ptr.mutability,
        star_token: ptr.star_token,
    }
}

fn reference_to_associated_ty(rf: TypeReference, generic_idents: &[&Ident]) -> TypeReference {
    TypeReference {
        and_token: rf.and_token,
        lifetime: rf.lifetime,
        mutability: rf.mutability,
        elem: Box::new(to_associated_ty(*rf.elem, generic_idents)),
    }
}
fn slice_to_associated_ty(slice: TypeSlice, generic_idents: &[&Ident]) -> TypeSlice {
    TypeSlice {
        bracket_token: slice.bracket_token,
        elem: Box::new(to_associated_ty(*slice.elem, generic_idents)),
    }
}

fn tuple_to_associated_ty(tup: TypeTuple, generic_idents: &[&Ident]) -> TypeTuple {
    let elems = tup
        .elems
        .into_iter()
        .map(|ty| to_associated_ty(ty, generic_idents))
        .collect();
    TypeTuple {
        paren_token: tup.paren_token,
        elems,
    }
}

fn arg_to_associated_ty(arg: GenericArgument, generic_idents: &[&Ident]) -> GenericArgument {
    match arg {
        GenericArgument::Type(ty) => GenericArgument::Type(to_associated_ty(ty, generic_idents)),
        GenericArgument::Lifetime(lt) => GenericArgument::Lifetime(lt),
        _ => panic!("unexpected item found in angular bracketed generic arguments"),
    }
}

fn ab_to_associated_ty(
    ab: AngleBracketedGenericArguments,
    generic_idents: &[&Ident],
) -> AngleBracketedGenericArguments {
    let args = ab
        .args
        .into_iter()
        .map(|arg| arg_to_associated_ty(arg, generic_idents))
        .collect();
    AngleBracketedGenericArguments {
        colon2_token: ab.colon2_token,
        lt_token: ab.lt_token,
        gt_token: ab.gt_token,
        args,
    }
}
fn parenthesized_to_associated_ty(
    par: ParenthesizedGenericArguments,
    generic_idents: &[&Ident],
) -> ParenthesizedGenericArguments {
    let inputs = par
        .inputs
        .into_iter()
        .map(|ty| to_associated_ty(ty, generic_idents))
        .collect();
    let output = return_type_to_associated_ty(par.output, generic_idents);
    ParenthesizedGenericArguments {
        paren_token: par.paren_token,
        inputs,
        output,
    }
}

fn path_segment_to_associated_ty(seg: PathSegment, generic_idents: &[&Ident]) -> PathSegment {
    let arguments = match seg.arguments {
        PathArguments::AngleBracketed(ab) => {
            PathArguments::AngleBracketed(ab_to_associated_ty(ab, generic_idents))
        }
        PathArguments::Parenthesized(par) => {
            PathArguments::Parenthesized(parenthesized_to_associated_ty(par, generic_idents))
        }
        x => x,
    };
    PathSegment {
        ident: seg.ident,
        arguments,
    }
}
fn nongeneric_path_to_associated_ty(path: Path, generic_idents: &[&Ident]) -> Path {
    let segments = path
        .segments
        .into_iter()
        .map(|seg| path_segment_to_associated_ty(seg, generic_idents))
        .collect();
    Path {
        leading_colon: path.leading_colon,
        segments,
    }
}

fn path_to_associated_ty(path: Path, generic_idents: &[&Ident]) -> Path {
    if path.segments.len() == 1 {
        let first_segment = path.segments.first().unwrap();
        if generic_idents.iter().any(|id| **id == first_segment.ident) {
            let mut segments = Punctuated::<PathSegment, syn::token::Colon2>::new();
            segments.push(PathSegment {
                ident: format_ident!("Self"),
                arguments: PathArguments::None,
            });
            segments.push(first_segment.clone());
            Path {
                leading_colon: None,
                segments,
            }
        } else {
            nongeneric_path_to_associated_ty(path, generic_idents)
        }
    } else {
        nongeneric_path_to_associated_ty(path, generic_idents)
    }
}

use syn::TraitBound;
pub fn bound_to_associated_ty(bound: TypeParamBound, generic_idents: &[&Ident]) -> TypeParamBound {
    match bound {
        TypeParamBound::Trait(tr) => TypeParamBound::Trait(TraitBound {
            paren_token: tr.paren_token,
            modifier: tr.modifier,
            lifetimes: tr.lifetimes,
            path: path_to_associated_ty(tr.path, generic_idents),
        }),
        x => x,
    }
}

fn type_path_to_associated_ty(tp: TypePath, generic_idents: &[&Ident]) -> TypePath {
    TypePath {
        qself: tp.qself,
        path: path_to_associated_ty(tp.path, generic_idents),
    }
}

pub fn to_associated_ty(ty: Type, generic_idents: &[&Ident]) -> Type {
    use Type::*;
    match ty {
        Array(ta) => Array(array_to_associated_ty(ta, generic_idents)),
        BareFn(bft) => BareFn(bare_fn_to_associated_ty(bft, generic_idents)),
        Group(gr) => Group(group_to_associated_ty(gr, generic_idents)),
        Paren(pa) => Paren(paren_to_associated_ty(pa, generic_idents)),
        Path(tp) => Path(type_path_to_associated_ty(tp, generic_idents)),
        Ptr(pt) => Ptr(ptr_to_associated_ty(pt, generic_idents)),
        Reference(tr) => Reference(reference_to_associated_ty(tr, generic_idents)),
        Slice(sl) => Slice(slice_to_associated_ty(sl, generic_idents)),
        Tuple(tup) => Tuple(tuple_to_associated_ty(tup, generic_idents)),
        _ => {
            unimplemented!("")
        }
    }
}
