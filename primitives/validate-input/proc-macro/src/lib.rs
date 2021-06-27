use proc_macro::TokenStream;
// use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Ident, Error, Data::Struct, DataStruct};

#[proc_macro_derive(ValidateInput)]
pub fn validate_input_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_validate_input_macro(&ast)
}

fn impl_validate_input_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let data = &ast.data;

	// panic!("info: {:?}", data);

	let data_struct_op = match data {
		Struct(value) => { Some(value) },
		_ => None,
	};

	let data_struct = data_struct_op.expect("Trait only applicable to Structs");
	println!("Hello, Macro! My name is {:?}!", data_struct.fields);

    let gen = quote! {
        impl ValidateInput for #name {
            fn validate(&self) {
                // panic!("Hello, Macro! My name is {:?}!", stringify!(#data_struct));
            }
        }
    };
    gen.into()
}
