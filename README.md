# Glow Plug
A simple testing macro for [diesel](https://github.com/diesel-rs/diesel) that will automatically create a new clean database and automatically apply migrations to simplify testing

This is heavily inspired by [sqlx's](https://github.com/launchbadge/sqlx) `#[sqlx::test]` macro and attempts to bring its usefulness to diesel.

## Example
```rust
#[glow_plug::test]
fn test_insert_something(mut conn: PgConnection) -> Result<()> {
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

## Installation
1. Add `glow_plug` as a dev dependency.
```
[dev-dependencies]
glow_plug = "the-version"
```
2. Setup migrations 
Internally it just uses `diesel_migrations` which requires the embeded migrations to be a `const` variable. By default the macro just assumes the migrations are available by using `crate::MIGRATIONS`. So in the root of your project you must do.

These are just reexported from `diesel_migrations` so if you already have the embeded migrations setup you can continue to just use those and remove the `#[cfg(test)]`
```rust
#[cfg(test)]
const MIGRATIONS: glow_plug::EmbeddedMigrations = glow_plug::embed_migrations!();
```
3. Make sure the `DATABASE_URL` in the `.env` file is set.
```
DATABASE_URL=postgres://...
```
4. Run the tests!
```
cargo test
```

## Contributing
PR's are welcome! If people find this useful would love some idea on how to make this more configurable for other setups since the current setup requires some additional things. e.g. `MIGRATIONS` and `.env`

## License
Copyright © 2024 Wes Wickwire. All rights reserved. Distributed under the MIT License.

[See the LICENSE file.](./LICENSE)