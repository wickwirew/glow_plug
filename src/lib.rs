extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn with_db(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let test_fn_name = &input.sig.ident;
    let test_fn_name_inner =
        syn::Ident::new(&format!("{}_inner", test_fn_name), test_fn_name.span());
    let block = &input.block;
    let inputs = &input.sig.inputs;

    // The type of connection, e.g. `PgConnection` or `MysqlConnection`
    let conn_type = match inputs.first() {
        Some(syn::FnArg::Typed(syn::PatType { ty, .. })) => ty,
        _ => panic!("Expected at least one input argument"),
    };

    // The temporary database name that the test will use
    let test_db_name = format!("{}_{}", test_fn_name, rand::random::<u32>());

    let gen = quote! {
        fn #test_fn_name_inner(#inputs) #block

        #[test]
        fn #test_fn_name() {
            use diesel::{Connection, RunQueryDsl};
            use diesel_migrations::MigrationHarness;

            dotenvy::dotenv().ok();

            let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

            let mut main_conn = <#conn_type>::establish(db_url.as_str())
                .expect("Failed to establish connection.");

            diesel::sql_query(format!("CREATE DATABASE {};", #test_db_name))
                .execute(&mut main_conn)
                .expect("Failed to execute query.");

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
            let result = std::panic::catch_unwind(|| {
                #test_fn_name_inner(conn);
            });

            diesel::sql_query(format!("DROP DATABASE {};", #test_db_name))
                .execute(&mut main_conn)
                .expect("Failed to execute query.");

            // If the test failed, rethrow
            if let Err(err) = result {
                std::panic::resume_unwind(err);
            }
        }
    };

    gen.into()
}
