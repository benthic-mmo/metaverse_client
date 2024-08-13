use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(IntoBoxed)]
pub fn derive_into_boxed(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = match &input.data {
        Data::Enum(data_enum) => {
            let variants = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                quote! {
                    Self::#variant_name(data) => data,
                }
            });

            quote! {
                impl #name {
                    pub fn into_boxed(self) -> Box<dyn PacketData> {
                        match self {
                            #(#variants)*
                        }
                    }
                }
            }
        }
        _ => panic!("`IntoBoxed` can only be derived for enums"),
    };

    TokenStream::from(expanded)
}
