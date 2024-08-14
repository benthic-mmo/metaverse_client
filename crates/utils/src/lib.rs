use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataEnum};

#[proc_macro_derive(IntoArc)]
pub fn derive_into_arc(input: TokenStream) -> TokenStream {
    // Parse the input token stream into a DeriveInput
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Match the input data to extract enum variants
    let expanded = match &input.data {
        Data::Enum(DataEnum { variants, .. }) => {
            // Generate the match arms for each enum variant
            let variants_match_arms = variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                quote! {
                    #name::#variant_name(boxed) => Arc::from(boxed),
                }
            });

            quote! {
                impl #name {
                    pub fn into_arc(self) -> Arc<dyn PacketData> {
                        match self {
                            #(#variants_match_arms)*
                        }
                    }
                }
            }
        }
        _ => panic!("`IntoArc` can only be derived for enums"),
    };

    TokenStream::from(expanded)
}

