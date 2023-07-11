extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse::Parser, parse_macro_input, parse_quote, ItemFn, ItemStruct};

#[proc_macro_attribute]
pub fn metadata(mut uid_field_name: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);

    if uid_field_name.is_empty() {
        uid_field_name = "_id".parse().unwrap();
    }

    let uid_field_name = parse_macro_input!(uid_field_name as Ident);

    let name = item_struct.ident.clone();
    let kind_name_str = name.to_string().to_ascii_lowercase();
    let kind = Ident::new(&kind_name_str, Span::call_site());
    let kind_fn = ItemFn::from(parse_quote!(fn #kind()->String {#kind_name_str.to_string()}));

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.extend(vec![
            syn::Field::parse_named
                .parse2(quote! {
                   #[serde(default)]
                   #[builder(default = bson::oid::ObjectId::new().to_string())]
                   pub #uid_field_name: String
                })
                .unwrap(),
            syn::Field::parse_named
                .parse2(quote! {
                    #[serde(default)]
                    #[builder(default = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())]
                    pub version: u64
                })
                .unwrap(),
            syn::Field::parse_named
                .parse2(quote! {
                    #[serde(default=#kind_name_str)]
                    #[builder(default = #kind_name_str.to_string())]
                    pub kind: String
                })
                .unwrap(),
        ]);
    }

    quote! {
        #[derive(Debug,Clone,serde::Deserialize,serde::Serialize,typed_builder::TypedBuilder)]
        #[builder(field_defaults(default))]
        pub #item_struct

        impl Object for #name {
            fn uid(&self) -> &str {&self.#uid_field_name}
            fn version(&self) -> u64 {self.version}
            fn kind(&self) -> &str {&self.kind}
            fn update_uid(&mut self, id: &str) { if self.#uid_field_name.is_empty() {self.#uid_field_name = id.to_string()}}
            fn update_version(&mut self,version:u64) { self.version = version }
        }

        #kind_fn
    }
    .into()
}
