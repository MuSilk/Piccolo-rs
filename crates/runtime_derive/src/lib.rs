//! 为 `runtime::function::framework::component::component::ComponentTrait` 生成默认 `as_any` / `as_any_mut` 实现。
//!
//! 用法：`use runtime_derive::ComponentTrait;` 后 `#[derive(ComponentTrait)]`。
//! 展开路径为 `::runtime::function::...::ComponentTrait`；在 `runtime` 包内需在 `lib.rs` 写 `extern crate self as runtime;`。

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ComponentTrait)]
pub fn derive_component_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::runtime::function::framework::component::component::ComponentTrait
            for #name #ty_generics #where_clause
        {
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                self
            }
        }
    };

    TokenStream::from(expanded)
}
