extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
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

    let test_db_name = syn::LitStr::new(
        &format!("{}_{}", test_fn_name, rand::random::<u32>()),
        test_fn_name.span(),
    );

    let gen = quote! {
        fn #test_fn_name_inner(#inputs) #block

        #[test]
        fn #test_fn_name() {
            use diesel::{Connection, RunQueryDsl};

            dotenvy::dotenv().ok();

            let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

            let mut main_conn = <#conn_type>::establish(db_url.as_str())
                .expect("Failed to establish connection.");

            diesel::sql_query(format!("CREATE DATABASE {};", #test_db_name))
                .execute(&mut main_conn)
                .expect("Failed to execute query.");

            let mut conn = <#conn_type>::establish(format!("postgresql://postgres:password@localhost:5432/{}", #test_db_name).as_str())
                .expect("Failed to establish connection.");

            let result = std::panic::catch_unwind(|| {
                #test_fn_name_inner(conn);
            });

            diesel::sql_query(format!("DROP DATABASE {};", #test_db_name))
                .execute(&mut main_conn)
                .expect("Failed to execute query.");

            if let Err(err) = result {
                std::panic::resume_unwind(err);
            }
        }
    };

    gen.into()
}
