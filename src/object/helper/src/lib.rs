extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use std::time::{SystemTime, UNIX_EPOCH};
use syn::{parse::Parser, parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn decorate(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);

    let mut uid = _attr.to_string();
    if uid == "" {
        uid = "_id".to_string();
    }

    let name = item_struct.ident.clone();
    let kind = name.clone().to_string();

    let meta = vec![
        syn::Field::parse_named
            .parse2(quote! {
               #[serde(rename(serialize = #uid, deserialize = #uid))]
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
        #[serde(deny_unknown_fields)]
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
            fn get_version(&mut self){
                self.version = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
            }
        }

        pub fn get_kind() -> String{
            #kind.to_string()
        }


    };

    ret.into()
}

#[proc_macro_derive(Object, attributes(helper))]
pub fn derive_object_fn(_item: TokenStream) -> TokenStream {
    "fn object() -> u32 { 42 }".parse().unwrap()
}
