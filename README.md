# sqlx-named-bind

[![Crates.io](https://img.shields.io/crates/v/sqlx-named-bind.svg)](https://crates.io/crates/sqlx-named-bind)
[![Documentation](https://docs.rs/sqlx-named-bind/badge.svg)](https://docs.rs/sqlx-named-bind)
[![License](https://img.shields.io/crates/l/sqlx-named-bind.svg)](https://github.com/AkiraKaneshiro/sqlx-named-bind#license)

A SQLx extension that provides named parameter binding with HRTB (Higher-Rank Trait Bounds) pattern, avoiding self-referential lifetime issues.

## Features

- ✨ **Named Placeholders**: Use `:param_name` instead of `?` in your SQL queries
- 🔒 **Type-Safe**: Full compile-time type checking with SQLx
- 🚀 **Zero Overhead**: Placeholder conversion happens at construction time
- 🔄 **Generic Executor**: Works with `MySqlPool`, `Transaction`, and any SQLx `Executor`
- 📦 **Strongly-Typed Results**: `PreparedQueryAs` provides type-safe query results via `FromRow`
- 🛡️ **Lifetime Safe**: HRTB pattern avoids self-referential lifetime issues

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sqlx = { version = "0.8", features = ["mysql", "runtime-tokio"] }
sqlx-named-bind = "0.1"
```

## Quick Start

```rust
use sqlx::MySqlPool;
use sqlx_named_bind::PreparedQuery;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = MySqlPool::connect("mysql://localhost/test").await?;

    let user_id = 42;
    let name = "John Doe";

    let mut query = PreparedQuery::new(
        "INSERT INTO users (id, name) VALUES (:id, :name)",
        |q, key| match key {
            ":id" => q.bind(user_id),
            ":name" => q.bind(name),
            _ => q,
        }
    )?;

    let result = query.execute(&pool).await?;
    println!("Inserted {} rows", result.rows_affected());

    Ok(())
}
```

## Examples

### Typed Query Results

```rust
use sqlx::{MySqlPool, FromRow};
use sqlx_named_bind::PreparedQueryAs;

#[derive(FromRow)]
struct User {
    id: i32,
    name: String,
    email: String,
}

async fn find_users(pool: &MySqlPool, min_age: i32) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let mut query = PreparedQueryAs::<User, _>::new(
        "SELECT id, name, email FROM users WHERE age >= :min_age",
        |q, key| match key {
            ":min_age" => q.bind(min_age),
            _ => q,
        }
    )?;

    Ok(query.fetch_all(pool).await?)
}
```

### Using with Transactions

```rust
use sqlx::{MySqlPool, Transaction, MySql};
use sqlx_named_bind::PreparedQuery;

async fn transfer_money(
    pool: &MySqlPool,
    from_id: i32,
    to_id: i32,
    amount: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx: Transaction<MySql> = pool.begin().await?;

    let mut debit = PreparedQuery::new(
        "UPDATE accounts SET balance = balance - :amount WHERE id = :id",
        |q, key| match key {
            ":amount" => q.bind(amount),
            ":id" => q.bind(from_id),
            _ => q,
        }
    )?;

    let mut credit = PreparedQuery::new(
        "UPDATE accounts SET balance = balance + :amount WHERE id = :id",
        |q, key| match key {
            ":amount" => q.bind(amount),
            ":id" => q.bind(to_id),
            _ => q,
        }
    )?;

    debit.execute(&mut *tx).await?;
    credit.execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}
```

### Optional Results

```rust
use sqlx::{MySqlPool, FromRow};
use sqlx_named_bind::PreparedQueryAs;

#[derive(FromRow)]
struct User {
    id: i32,
    name: String,
}

async fn find_user_by_email(
    pool: &MySqlPool,
    email: &str,
) -> Result<Option<User>, Box<dyn std::error::Error>> {
    let mut query = PreparedQueryAs::<User, _>::new(
        "SELECT id, name FROM users WHERE email = :email",
        |q, key| match key {
            ":email" => q.bind(email),
            _ => q,
        }
    )?;

    Ok(query.fetch_optional(pool).await?)
}
```

## How It Works

The library uses a three-step approach to avoid self-referential lifetime issues:

1. **Parse**: Extract named placeholders (`:name`) and convert SQL to use positional placeholders (`?`)
2. **Store**: Keep the converted SQL, placeholder order, and binder function separately
3. **Execute**: Construct a fresh SQLx `Query` on each execution with the correct lifetime

This approach leverages HRTB (Higher-Rank Trait Bounds) to ensure the binder function works with any lifetime, making the API both safe and flexible.

### Why HRTB?

Without HRTB, you'd encounter self-referential lifetime issues:

```rust
// ❌ This doesn't work - self-referential lifetime
struct BadQuery<'a> {
    query: Query<'a, MySql>,  // 'a refers to data inside BadQuery
}
```

With HRTB, we defer the lifetime decision to call-site:

```rust
// ✅ This works - lifetime chosen at each call
where F: for<'q> FnMut(Query<'q, MySql>, &str) -> Query<'q, MySql>
```

## API Documentation

### `PreparedQuery`

For queries that execute but don't return rows (`INSERT`, `UPDATE`, `DELETE`).

**Methods:**
- `new(template, binder)` - Create a new prepared query
- `execute(executor)` - Execute the query and return `MySqlQueryResult`

### `PreparedQueryAs<R>`

For queries that return typed rows (`SELECT`).

**Methods:**
- `new(template, binder)` - Create a new prepared query
- `fetch_all(executor)` - Fetch all matching rows
- `fetch_one(executor)` - Fetch exactly one row (error if 0 or >1)
- `fetch_optional(executor)` - Fetch at most one row (returns `Option<R>`)

## Limitations

- Currently only supports MySQL (PostgreSQL and SQLite support planned)
- Placeholder names must match `[a-zA-Z0-9_]+`
- All placeholders in the SQL must be handled by the binder function

## Comparison with Alternatives

| Feature | sqlx-named-bind | SQLx native | Other libraries |
|---------|----------------|-------------|-----------------|
| Named parameters | ✅ `:name` | ❌ `?` only | ✅ Varies |
| Type safety | ✅ Full | ✅ Full | ⚠️ Varies |
| Lifetime safety | ✅ HRTB | ✅ Native | ⚠️ Varies |
| Generic executor | ✅ Yes | ✅ Yes | ❌ Usually pool-only |
| Runtime overhead | ✅ Zero | ✅ Zero | ⚠️ Some have overhead |

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

This library was inspired by the need for named parameter binding in SQLx while maintaining the same level of type safety and performance. Special thanks to the SQLx team for creating an excellent async SQL toolkit.
