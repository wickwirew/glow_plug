# Glow Plug
A simple testing macro `#[glow_plug::test]` for [diesel](https://github.com/diesel-rs/diesel).

Glow Plug is a Rust testing library designed to simplify database testing with Diesel. It provides a macro that automatically sets up a new, clean database and applies migrations for each test, ensuring test isolation and reliability. With Glow Plug, you can confidently write tests that interact with a real database without worrying about tests interfering with each other or having hidden dependencies.

## Example
The connection provided to the test is always a clean, empty database.

#### Example 1.
```rust
#[glow_plug::test]
fn test_the_database_is_empty(mut conn: PgConnection) {
    let results = posts
        .filter(published.eq(true))
        .select(Post::as_select())
        .load(conn)
        .expect("Error loading posts");

    assert_eq!(results.len(), 0);
}
```

#### Example 2.
Also can handle test that return results.
```rust
#[glow_plug::test]
fn test_insert_user(mut conn: PgConnection) -> Result<()> {
    let user = NewUser {
        id: "12345",
        ...
    };

    diesel::insert_into(users)
        .values(user)
        .execute(&mut conn)?;

    let user = users
        .filter(user_account::id.eq("12345"))
        .first::<User>(&mut conn)?;

    assert_eq!(user.id, "12345");
}
```

#### Async Example
Enabling the `tokio` feature will allow you to run `async` tests.
```rust
#[glow_plug::test]
async fn test_insert_user(mut conn: PgConnection) -> Result<()> {
    let result = something_async().await?;
    assert(result);
}
```

## Installation
#### 1. Add `glow_plug` as a dev dependency.
Find the latest version on [crates.io](https://crates.io).
```
[dev-dependencies]
glow_plug = "the-version"
```
#### 2. Setup migrations 

Internally it just uses `diesel_migrations` which requires the embeded migrations to be a `const` variable. By default the macro just assumes the migrations are available by using `crate::MIGRATIONS`. So in the root of your project you must declare the `MIGRATIONS`.

These are just reexported from `diesel_migrations` so if you already have the embeded migrations setup you can continue to just use those and remove the `#[cfg(test)]`
```rust
#[cfg(test)]
const MIGRATIONS: glow_plug::EmbeddedMigrations = glow_plug::embed_migrations!();
```
#### 3. Make sure the `DATABASE_URL` in the `.env` file is set.
```
DATABASE_URL=postgres://...
```
#### 4. Run the tests!
```
cargo test
```

## Contributing
PR's are welcome! If people find this useful would love some ideas on how to make this more configurable for other setups since the current setup requires some additional things and the macro just assumes where the `MIGRATIONS` are and that you use `.env`.

## Credits
This is heavily inspired by [sqlx's](https://github.com/launchbadge/sqlx) `#[sqlx::test]` macro and attempts to bring its usefulness to diesel. So credit to them for the idea!

## License
Copyright © 2024 Wes Wickwire. All rights reserved. Distributed under the MIT License.

[See the LICENSE file.](./LICENSE)
