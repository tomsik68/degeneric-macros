use super::generics::AssociatedType;
use quote::{quote, ToTokens};

pub fn emit_dynamize(associated_types: &[AssociatedType]) -> impl ToTokens {
    let ident = associated_types.iter().map(|at| &at.0.ident);
    let bounds = associated_types.iter().map(|at| &at.0.bounds);

    quote! {
        #[dynamize::dynamize]
        #(
            #[convert = |value_to_convert: &Self::#ident| -> &( #bounds ) {
                value_to_convert
            }]
            #[convert = |value_to_convert: &mut Self::#ident| -> &mut ( #bounds ) {
                value_to_convert
            }]
        )*
    }
}
