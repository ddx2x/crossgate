extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn decorate(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(args as syn::parse::Nothing);

    let name = item_struct.ident.clone();
    let kind = name.clone().to_string();

    let meta = vec![
        syn::Field::parse_named
            .parse2(quote! {
               #[serde(rename(serialize = "_id", deserialize = "_id"))]
               #[serde(default)]
               pub uid: String
            })
            .unwrap(),
        syn::Field::parse_named
            .parse2(quote! {
                #[serde(default)]
                pub version: u64
            })
            .unwrap(),
        syn::Field::parse_named
            .parse2(quote! {
                #[serde(default="get_kind")]
                pub kind: String
            })
            .unwrap(),
    ];

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.extend(meta);
    }

    let ret = quote! {
        #[derive(Debug,Clone,serde::Deserialize,serde::Serialize)]
        pub #item_struct

        impl Object for #name {
            fn uid(&self) -> &str {
                &self.uid
            }
            fn version(&self) -> u64 {
                self.version
            }
            fn kind(&self) -> &str {
                &self.kind
            }
            fn generate(&mut self, f: fn() -> String){
                self.uid = f()
            }
        }

        pub fn get_kind() -> String{
            #kind.to_string()
        }


    };

    ret.into()
}
