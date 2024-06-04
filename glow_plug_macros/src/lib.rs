extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, ItemFn, Signature, Type};

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    let conn_type = get_conn_type(&func);
    let db_name = test_db_name(&func);
    let (sig, sig_inner) = get_signatures(&func);
    let inner_ident = &sig_inner.ident;
    let inner_block = &func.block;

    let run_test = if func.sig.asyncness.is_some() {
        quote! {
            use glow_plug::FutureExt;
            let result = #inner_ident(conn).catch_unwind().await;
        }
    } else {
        quote! {
            let result = std::panic::catch_unwind(|| {
                #inner_ident(conn)
            });
        }
    };

    let test_macro = if func.sig.asyncness.is_some() {
        quote! { #[glow_plug::tokio::test] }
    } else {
        quote! { #[test] }
    };

    quote! {
        #test_macro
        #sig {
            #sig_inner #inner_block

            use glow_plug::{Connection, RunQueryDsl};
            use glow_plug::MigrationHarness;

            glow_plug::dotenvy::dotenv().ok();

            let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

            let mut main_conn = <#conn_type>::establish(db_url.as_str())
                .expect("Failed to establish connection.");

            diesel::sql_query(format!("CREATE DATABASE {};", #db_name))
                .execute(&mut main_conn)
                .expect("Failed to create database.");

            // We cannot just change the `conn` to point to the new database since not
            // all databases don't support a `USE database` command. So just connect again
            let mut conn = <#conn_type>::establish(format!("{}/{}", db_url, #db_name).as_str())
                .expect("Failed to establish connection.");

            // Run the migrations, there may be a better way to do this. Right now it currently
            // requires the migrations to be a const variable in the crate root also the user
            // needs to manually add the `diesel_migrations` dependency with the correct features.
            conn.run_pending_migrations(crate::MIGRATIONS)
                .expect("Failed to run migrations");

            // Make sure to catch the panic, so we can drop the database even on failure.
            #run_test

            diesel::sql_query(format!("DROP DATABASE {};", #db_name))
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

fn get_signatures(func: &ItemFn) -> (Signature, Signature) {
    let mut sig = func.sig.clone();
    let mut sig_inner = func.sig.clone();

    // Remove the inputs of the outer function
    sig.inputs = Punctuated::new();

    // Rename the inner function
    sig_inner.ident = syn::Ident::new(&format!("{}_inner", sig.ident), sig.ident.span());

    (sig, sig_inner)
}

// Gets the type of connection, e.g. `PgConnection` or `MysqlConnection`
fn get_conn_type(func: &ItemFn) -> &Box<Type> {
    let args = &func.sig.inputs;

    match args.first() {
        Some(syn::FnArg::Typed(syn::PatType { ty, .. })) => ty,
        _ => panic!("Expected at least one input argument"),
    }
}

fn test_db_name(func: &ItemFn) -> String {
    let name = func.sig.ident.to_string();
    format!("{}_{}", name, rand::random::<u32>())
}
