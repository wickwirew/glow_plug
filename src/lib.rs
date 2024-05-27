extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn with_db(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let test_fn_name = &input.sig.ident;
    let test_fn_name_inner =
        syn::Ident::new(&format!("{}_inner", test_fn_name), test_fn_name.span());
    let block = &input.block;
    let inputs = &input.sig.inputs;

    let conn_type = match inputs.first() {
        Some(syn::FnArg::Typed(syn::PatType { ty, .. })) => ty,
        _ => panic!("Expected at least one input argument"),
    };

    let gen = quote! {
        fn #test_fn_name_inner(#inputs) #block

        #[test]
        fn #test_fn_name() {
            use diesel::{Connection, RunQueryDsl};

            let mut main_conn = <#conn_type>::establish("postgresql://postgres:password@localhost:5432")
                .expect("Failed to establish connection.");

            diesel::sql_query("CREATE DATABASE test_db;")
                .execute(&mut main_conn)
                .expect("Failed to execute query.");

            let mut conn = <#conn_type>::establish("postgresql://postgres:password@localhost:5432/test_db")
                .expect("Failed to establish connection.");

            #test_fn_name_inner(conn);

            diesel::sql_query("DROP DATABASE test_db;")
                .execute(&mut main_conn)
                .expect("Failed to execute query.");
        }
    };

    gen.into()
}
