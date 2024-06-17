use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{Data, DeriveInput};

pub(crate) fn param_enum_inner(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let n = match &ast.data {
        Data::Enum(v) => v.variants.iter().try_fold(0usize, |acc, _v| {
            Ok::<usize, syn::Error>(acc + 1usize)
        })?,
        _ => return Err(syn::Error::new(Span::call_site(), "This macro only supports enums.")),
    };

    // Used in the quasi-quotation below as `#name`
    let name = &ast.ident;

    // Helper is provided for handling complex generic types correctly and effortlessly
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    Ok(quote! {
        // Implementation
        impl #impl_generics Into<usize> for #name #ty_generics #where_clause {
            fn into(self) -> usize {
                self as usize
            }
        }
        impl #impl_generics mortalsim_math_routines::params::Param for #name #ty_generics #where_clause {
            const COUNT: usize = #n;
        }
    })
}
