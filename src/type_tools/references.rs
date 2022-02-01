use syn::spanned::Spanned;
use syn::Error;
use syn::Token;
use syn::Type;
use syn::TypeParen;
use syn::TypePtr;
use syn::TypeReference;

pub fn can_be_made_mutable(ty: &Type) -> bool {
    match ty {
        Type::Reference(rf) => rf.mutability.is_some(),
        Type::Ptr(tp) => tp.mutability.is_some(),
        _ => true,
    }
}

pub fn make_reference(ty: Type, mutability: Option<Token![mut]>) -> Result<Type, Error> {
    match ty {
        Type::Reference(rf) => Ok(Type::Reference(TypeReference { mutability, ..rf })),
        x @ Type::Never(_) => Ok(x),
        Type::Ptr(ptr) => Ok(Type::Ptr(TypePtr { mutability, ..ptr })),
        Type::Paren(tp) => Ok(Type::Paren(TypeParen {
            elem: Box::new(make_reference(*tp.elem, mutability)?),
            ..tp
        })),
        tt @ Type::Array(_)
        | tt @ Type::BareFn(_)
        | tt @ Type::Group(_)
        | tt @ Type::Infer(_)
        | tt @ Type::Macro(_)
        | tt @ Type::Path(_)
        | tt @ Type::Slice(_)
        | tt @ Type::TraitObject(_)
        | tt @ Type::Tuple(_)
        | tt @ Type::Verbatim(_)
        | tt @ Type::ImplTrait(_) => Ok(Type::Reference(TypeReference {
            and_token: Default::default(),
            elem: Box::new(tt),
            lifetime: None,
            mutability,
        })),
        x => Err(Error::new(
            x.span(),
            "degeneric failed to convert the type to reference",
        )),
    }
}
