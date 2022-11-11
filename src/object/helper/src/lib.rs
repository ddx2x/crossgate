extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
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
            fn uid(&self) -> &str {&self.uid}
            fn version(&self) -> u64 {self.version}
            fn kind(&self) -> &str {&self.kind}
            fn set_uid(&mut self,id:&str){self.uid = id.to_string()}
            fn set_version(&mut self,version:u64){self.version = version}
        }

        pub fn get_kind() -> String{
            #kind.to_string()
        }


    };

    ret.into()
}
