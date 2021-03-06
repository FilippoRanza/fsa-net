extern crate proc_macro;

use proc_macro::TokenStream;
use quote;
use syn;
use syn::parse::Parser;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn add_location(_: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input);
    let ast = add_location_fields(ast);
    add_location_funcitions(ast)
}

fn add_location_fields(mut ast: syn::DeriveInput) -> syn::DeriveInput {
    if let syn::Data::Struct(data) = &mut ast.data {
        if let syn::Fields::Named(fields) = &mut data.fields {
            fields.named.push(
                syn::Field::parse_named
                    .parse2(quote::quote! {
                        __begin__ : usize
                    })
                    .unwrap(),
            );
            fields.named.push(
                syn::Field::parse_named
                    .parse2(quote::quote! {
                        __end__: usize
                    })
                    .unwrap(),
            );
        }
    }
    ast
}

fn add_location_funcitions(ast: syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;

    let gen = quote::quote! {
        #ast
        impl#generics #name#generics {
            pub fn get_location(&self) -> (usize, usize) {
                (self.__begin__, self.__end__)
            }
            pub fn set_location(mut self, begin: usize, end: usize) -> Self {
                self.__begin__ = begin;
                self.__end__ = end;
                self
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(DefaultBuilder)]
pub fn default_builder(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input);
    impl_default_builder(&ast)
}

fn impl_default_builder(ast: &syn::DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;

    let params = default_builder_parameters(&ast);

    let fields = default_builder_fields(&ast);

    let generics = &ast.generics;
    let gen = quote::quote! {
        impl#generics  #struct_name#generics{
            pub fn new(#(#params),*) -> Self {
                Self {
                    #(#fields),*
                }
            }
        }
    };

    gen.into()
}

fn default_builder_parameters<'a>(
    ast: &'a syn::DeriveInput,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    match &ast.data {
        syn::Data::Struct(struct_data) => struct_data
            .fields
            .iter()
            .filter_map(|field| {
                if let Some(name) = &field.ident {
                    if !is_automatic_field(name) {
                        Some((name, &field.ty))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .map(|(name, typ)| quote::quote! {#name : #typ}),
        _ => {
            panic!("Only struct can derive `DefaultBuilder`")
        }
    }
}

fn default_builder_fields<'a>(
    ast: &'a syn::DeriveInput,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    match &ast.data {
        syn::Data::Struct(struct_data) => struct_data
            .fields
            .iter()
            .filter_map(|field| {
                if let Some(name) = &field.ident {
                    let output = if is_automatic_field(name) {
                        FieldType::Automatic(name)
                    } else {
                        FieldType::User(name)
                    };
                    Some(output)
                } else {
                    None
                }
            })
            .map(|filed| match filed {
                FieldType::Automatic(name) => quote::quote! { #name : Default::default()},
                FieldType::User(name) => quote::quote! {#name},
            }),
        _ => {
            panic!("Only struct can derive `DefaultBuilder`")
        }
    }
}

fn is_automatic_field(ident: &syn::Ident) -> bool {
    let name = ident.to_string();
    name.starts_with("__") && name.ends_with("__")
}

enum FieldType<'a> {
    User(&'a syn::Ident),
    Automatic(&'a syn::Ident),
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
