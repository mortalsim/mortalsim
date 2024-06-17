extern crate proc_macro;

mod macros;

use proc_macro2::TokenStream;
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
///
#[proc_macro_derive(ParamEnum)]
pub fn param_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let toks =
        macros::param_enum::param_enum_inner(&ast).unwrap_or_else(|err| err.to_compile_error());
    debug_print_generated(&ast, &toks);
    toks.into()
}
