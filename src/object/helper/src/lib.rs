extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse::Parser, parse_macro_input, parse_quote, ItemFn, ItemStruct};

#[proc_macro_attribute]
pub fn decorate(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);

    let mut uid = _attr.to_string();
    if uid == "" {
        uid = "_id".to_string();
    }

    let name = item_struct.ident.clone();
    let kind_name_str = name.to_string().to_ascii_lowercase();
    let kind = Ident::new(&kind_name_str, Span::call_site());
    let kind_fn = ItemFn::from(parse_quote!(fn #kind()->String {#kind_name_str.to_string()}));

    let metadata_fields = vec![
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
                #[serde(default=#kind_name_str)]
                pub kind: String
            })
            .unwrap(),
    ];

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.extend(metadata_fields);
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

        #kind_fn
    };

    ret.into()
}
