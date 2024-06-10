extern crate proc_macro;

mod macros;

use proc_macro2::{Span, TokenStream};
use std::env;
use syn::DeriveInput;


fn debug_print_generated(ast: &DeriveInput, toks: &TokenStream) {
    let debug = env::var("MORTALSIM_MACROS_DEBUG");
    if let Ok(s) = debug {
        if s == "1" {
            println!("{}", toks);
        }

        if ast.ident == s {
            println!("{}", toks);
        }
    }
}

/// Implements ConstantParam for the given Enum
///
/// For a given enum generates implementations of:
/// - `Into<usize>`
/// - `mortalsim_math_routines::params::Param`
/// - `mortalsim_math_routines::params::ConstantParam`
///
#[proc_macro_derive(ConstantParamEnum)]
pub fn constant_param_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let p_ty = syn::Ident::new("ConstantParam", Span::call_site());
    let toks =
        macros::param_enum::param_enum_inner(&ast, &p_ty).unwrap_or_else(|err| err.to_compile_error());
    debug_print_generated(&ast, &toks);
    toks.into()
}

/// Implements AssignmentParam for the given Enum
///
/// For a given enum generates implementations of:
/// - `Into<usize>`
/// - `mortalsim_math_routines::params::Param`
/// - `mortalsim_math_routines::params::AssignmentParam`
///
#[proc_macro_derive(AssignmentParamEnum)]
pub fn assignment_param_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let p_ty = syn::Ident::new("AssignmentParam", Span::call_site());
    let toks =
        macros::param_enum::param_enum_inner(&ast, &p_ty).unwrap_or_else(|err| err.to_compile_error());
    debug_print_generated(&ast, &toks);
    toks.into()
}

/// Implements RateBoundParam for the given Enum
///
/// For a given enum generates implementations of:
/// - `Into<usize>`
/// - `mortalsim_math_routines::params::Param`
/// - `mortalsim_math_routines::params::RateBoundParam`
///
#[proc_macro_derive(RateBoundParamEnum)]
pub fn rate_bound_param_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let p_ty = syn::Ident::new("RateBoundParam", Span::call_site());
    let toks =
        macros::param_enum::param_enum_inner(&ast, &p_ty).unwrap_or_else(|err| err.to_compile_error());
    debug_print_generated(&ast, &toks);
    toks.into()
}
