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
```
#[cfg(test)]
const MIGRATIONS: glow_plug::EmbeddedMigrations = glow_plug::embed_migrations!();
```
Note: These are just reexported from `diesel_migrations` so if you already have the embeded migrations setup you can continue to just use those and remove the `#[cfg(test)]`

Also it would be nice if this was more customizable, so if anyone has any other usecases that require a different setup I'd be happy to accept a PR or an idea on what else to add.
3. Make sure the `DATABASE_URL` in the `.env` file is set.
```
DATABASE_URL=postgres://...
```
4. Run the tests!
```
cargo test
```
