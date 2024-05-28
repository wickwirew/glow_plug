# Glow Plug
A testing macro for [diesel](https://github.com/diesel-rs/diesel) that will automatically create a new clean database and automatically apply migrations to simplify testing

This is heavily inspired by [sqlx's](https://github.com/launchbadge/sqlx) `sqlx::test` macro and attempts to bring its usefulness to diesel.

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
        .first::<User>(&mut conn)
        .unwrap();

    assert_eq!(user.first_name, "Test");
}

```

## Installation
1. Add `glow_plug` as a dev dependency.
```
[dev-dependencies]
glow_plug = "the-version"
```
2. Setup migrations 
Internally it just uses `diesel_migrations` which requires the embeded migrations to be a `const` variable. By default the macro just assumes the migrations are inavailable by `crate::MIGRATIONS`. So in the root of your project just do
```
#[cfg(test)]
const MIGRATIONS: glow_plug::EmbeddedMigrations = glow_plug::embed_migrations!();
```
Note: These are just reexported from `diesel_migrations`
