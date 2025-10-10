use proc_macro::{TokenStream};
use quote::{format_ident, quote};
use syn::{Data};


#[proc_macro_derive(ReflectFields)]
pub fn reflect_fields_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_reflect_macro(&ast, true)
}

#[proc_macro_derive(ReflectWhiteListFields, attributes(meta))]
pub fn reflect_white_list_fields_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_reflect_macro(&ast, false)
}

fn impl_reflect_macro(ast: &syn::DeriveInput, all_fields: bool) -> TokenStream {

    let name = &ast.ident;
    let Data::Struct(st) = &ast.data else {
        panic!("Reflect can only be derived for structs");
    };

    let struct_type_st_operator = format_ident!("Type{}Operator", name);

    let mut struct_type_st_operator_impl = quote! {
        fn get_class_name() -> &'static str {
            stringify!(#name)
        }
    };
    let mut method_type_wrapper_register_st_ast = quote!{};

    for (_idx, field) in st.fields.iter().enumerate() {
        let meta_attr = field.attrs.iter().find(|attr| attr.path().is_ident("meta"));
        if ! all_fields && meta_attr.is_none() {
            continue;
        }
        let (field_id, field_type) = (&field.ident, &field.ty);
        if field_id.is_none() {
            panic!("Unnamed fields are not supported in Reflect derive macro");
        }
        let field_id = field_id.as_ref().unwrap();
        let method_get_field_name_id = format_ident!("get_field_name_{}", field_id);
        let method_get_field_type_name_id = format_ident!("get_field_type_name_{}", field_id);
        let method_set_id = format_ident!("set_{}", field_id);
        let method_get_id = format_ident!("get_{}", field_id);
        let method_is_array_id = format_ident!("is_array_{}", field_id);
        struct_type_st_operator_impl.extend(quote! {
            fn #method_get_field_name_id() -> &'static str {
                stringify!(#field_id)
            }
            fn #method_get_field_type_name_id() -> &'static str {
                stringify!(#field_type)
            }
            fn #method_set_id(instance: *mut std::os::raw::c_void, x: *const std::os::raw::c_void) {
                unsafe{
                    (&mut *(instance as *mut #name)).#field_id = (*(x as *const #field_type)).clone();
                }
            }
            fn #method_get_id(instance: *const std::os::raw::c_void) -> *const std::os::raw::c_void {
                unsafe{
                    let field_ref = &(*(instance as *const #name)).#field_id;
                    field_ref as *const #field_type as *const std::os::raw::c_void
                }
            }
            fn #method_is_array_id(instance: *const std::os::raw::c_void) -> bool {
                false
            }
        });
        method_type_wrapper_register_st_ast.extend(quote! {
            let field_function_tuple = (
                #struct_type_st_operator::#method_set_id as reflection::reflection::SetFunction,
                #struct_type_st_operator::#method_get_id as reflection::reflection::GetFunction,
                #struct_type_st_operator::get_class_name as reflection::reflection::GetNameFunction,
                #struct_type_st_operator::#method_get_field_name_id as reflection::reflection::GetNameFunction,
                #struct_type_st_operator::#method_get_field_type_name_id as reflection::reflection::GetNameFunction,
                #struct_type_st_operator::#method_is_array_id as reflection::reflection::GetBoolFunction,
            );
            reflection::reflection::TypeMetaRegisterInterface::register_to_field_map(stringify!(#name), field_function_tuple);
        });
    }

    let struct_type_name_operator_ast = quote! {
        struct #struct_type_st_operator {

        } 
        impl #struct_type_st_operator {
            #struct_type_st_operator_impl
        }
    };

    method_type_wrapper_register_st_ast.extend(quote! {
        reflection::reflection::TypeMetaRegisterInterface::register_to_class_map(stringify!(#name), ());
    });


    let method_type_wrapper_register_name = format_ident!("type_wrapper_register_{}", name);
    let method_type_wrapper_register_name = ident_to_snake_case(method_type_wrapper_register_name);
    let method_type_wrapper_register_name_ast = quote! {
        fn #method_type_wrapper_register_name() {
            #method_type_wrapper_register_st_ast
        }
    };

    let fn_type_wrapper_register_name = format_ident!("fn_type_wrapper_register_{}", name);
    let fn_type_wrapper_register_name = ident_to_upper_case(fn_type_wrapper_register_name);
    let generated_code = quote! {
        #struct_type_name_operator_ast
        #method_type_wrapper_register_name_ast
        #[linkme::distributed_slice(reflection::reflection_register::REFLECT_REGISTER_FUNCTION_LIST)]
        static #fn_type_wrapper_register_name: fn() = #method_type_wrapper_register_name as fn();
    };
    generated_code.into()
}

fn ident_to_upper_case(ident: syn::Ident) -> syn::Ident {

    syn::Ident::new(
        &ident.to_string().to_uppercase(),
        ident.span()
    )
}
fn ident_to_snake_case(ident: syn::Ident) -> syn::Ident {
    syn::Ident::new(
        &ident.to_string().to_lowercase(),
        ident.span(),
    )
}