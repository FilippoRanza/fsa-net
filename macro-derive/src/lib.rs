extern crate proc_macro;

use proc_macro::TokenStream;
use quote;
use syn;
use syn::parse_macro_input;

#[proc_macro_derive(IntoNameError)]
pub fn add_from_implement(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input);
    impl_from_error(&ast)
}

fn impl_from_error(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;

    let gen = quote::quote! {
        impl#generics From<#name#generics> for NameError#generics {
            fn from(err: #name#generics) -> NameError {
                NameError::#name(err)
            }
        }
    };
    println!("{}", gen);
    gen.into()
}


