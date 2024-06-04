extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let test_fn_name = &input.sig.ident;
    let test_fn_name_inner = syn::Ident::new(
        &format!("{}_glow_plug_inner", test_fn_name),
        test_fn_name.span(),
    );
    let return_type = &input.sig.output;
    let block = &input.block;
    let inputs = &input.sig.inputs;

    // The type of connection, e.g. `PgConnection` or `MysqlConnection`
    let conn_type = match inputs.first() {
        Some(syn::FnArg::Typed(syn::PatType { ty, .. })) => ty,
        _ => panic!("Expected at least one input argument"),
    };

    // The temporary database name that the test will use
    let test_db_name = format!("{}_{}", test_fn_name, rand::random::<u32>());

    let is_async = input.sig.asyncness.is_some();

    let async_modifier = if is_async {
        quote! { async }
    } else {
        quote! {}
    };

    let test_macro = if is_async {
        quote! { #[glow_plug::tokio::test] }
    } else {
        quote! { #[test] }
    };

    let run_test = if is_async {
        quote! {
            use glow_plug::FutureExt;
            let result = #test_fn_name_inner(conn).catch_unwind().await;
        }
    } else {
        quote! {
            let result = std::panic::catch_unwind(|| {
                #test_fn_name_inner(conn)
            });
        }
    };

    quote! {
        #test_macro
        #async_modifier fn #test_fn_name() #return_type {
            #async_modifier fn #test_fn_name_inner(#inputs) #return_type #block

            use glow_plug::{Connection, RunQueryDsl};
            use glow_plug::MigrationHarness;

            glow_plug::dotenvy::dotenv().ok();

            let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

            let mut main_conn = <#conn_type>::establish(db_url.as_str())
                .expect("Failed to establish connection.");

            diesel::sql_query(format!("CREATE DATABASE {};", #test_db_name))
                .execute(&mut main_conn)
                .expect("Failed to create database.");

            // We cannot just change the `conn` to point to the new database since not
            // all databases don't support a `USE database` command. So just connect again
            let mut conn = <#conn_type>::establish(format!("{}/{}", db_url, #test_db_name).as_str())
                .expect("Failed to establish connection.");

            // Run the migrations, there may be a better way to do this. Right now it currently
            // requires the migrations to be a const variable in the crate root also the user
            // needs to manually add the `diesel_migrations` dependency with the correct features.
            conn.run_pending_migrations(crate::MIGRATIONS)
                .expect("Failed to run migrations");

            // Make sure to catch the panic, so we can drop the database even on failure.
            #run_test

            diesel::sql_query(format!("DROP DATABASE {};", #test_db_name))
                .execute(&mut main_conn)
                .expect("Failed to drop database.");

            match result {
                Ok(val) => val,
                Err(err) => std::panic::resume_unwind(err),
            }
        }
    }
    .into()
}
