//! # sqlx-named-bind
//!
//! A SQLx extension that provides named parameter binding with HRTB (Higher-Rank Trait Bounds) pattern,
//! avoiding self-referential lifetime issues.
//!
//! ## Features
//!
//! - **Named Placeholders**: Use `:param_name` instead of `?` in your SQL queries
//! - **HRTB Pattern**: Avoids self-referential lifetime issues through proper use of Higher-Rank Trait Bounds
//! - **Generic Executor Support**: Works with `MySqlPool`, `Transaction`, and any SQLx `Executor`
//! - **Type-Safe Results**: `PreparedQueryAs` provides strongly-typed query results via `FromRow`
//! - **Zero Runtime Overhead**: Placeholder conversion happens at query construction time
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! sqlx = { version = "0.8", features = ["mysql", "runtime-tokio"] }
//! sqlx-named-bind = "0.1"
//! ```
//!
//! ## Examples
//!
//! ### Basic Query Execution
//!
//! ```rust,no_run
//! use sqlx::MySqlPool;
//! use sqlx_named_bind::PreparedQuery;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let pool = MySqlPool::connect("mysql://localhost/test").await?;
//!
//! let user_id = 42;
//! let name = "John Doe";
//!
//! let mut query = PreparedQuery::new(
//!     "INSERT INTO users (id, name) VALUES (:id, :name)",
//!     |q, key| match key {
//!         ":id" => q.bind(user_id),
//!         ":name" => q.bind(name),
//!         _ => q,
//!     }
//! )?;
//!
//! let result = query.execute(&pool).await?;
//! println!("Inserted {} rows", result.rows_affected());
//! # Ok(())
//! # }
//! ```
//!
//! ### Typed Query Results
//!
//! ```rust,no_run
//! use sqlx::{MySqlPool, FromRow};
//! use sqlx_named_bind::PreparedQueryAs;
//!
//! #[derive(FromRow)]
//! struct User {
//!     id: i32,
//!     name: String,
//!     email: String,
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let pool = MySqlPool::connect("mysql://localhost/test").await?;
//! let min_age = 18;
//!
//! let mut query = PreparedQueryAs::<User, _>::new(
//!     "SELECT id, name, email FROM users WHERE age >= :min_age",
//!     |q, key| match key {
//!         ":min_age" => q.bind(min_age),
//!         _ => q,
//!     }
//! )?;
//!
//! let users: Vec<User> = query.fetch_all(&pool).await?;
//! for user in users {
//!     println!("{}: {}", user.name, user.email);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Using with Transactions
//!
//! ```rust,no_run
//! use sqlx::{MySqlPool, Transaction, MySql};
//! use sqlx_named_bind::PreparedQuery;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let pool = MySqlPool::connect("mysql://localhost/test").await?;
//! let mut tx: Transaction<MySql> = pool.begin().await?;
//!
//! let mut query1 = PreparedQuery::new(
//!     "UPDATE accounts SET balance = balance - :amount WHERE id = :from_id",
//!     |q, key| match key {
//!         ":amount" => q.bind(100),
//!         ":from_id" => q.bind(1),
//!         _ => q,
//!     }
//! )?;
//!
//! let mut query2 = PreparedQuery::new(
//!     "UPDATE accounts SET balance = balance + :amount WHERE id = :to_id",
//!     |q, key| match key {
//!         ":amount" => q.bind(100),
//!         ":to_id" => q.bind(2),
//!         _ => q,
//!     }
//! )?;
//!
//! query1.execute(&mut *tx).await?;
//! query2.execute(&mut *tx).await?;
//!
//! tx.commit().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Optional Results
//!
//! ```rust,no_run
//! use sqlx::{MySqlPool, FromRow};
//! use sqlx_named_bind::PreparedQueryAs;
//!
//! #[derive(FromRow)]
//! struct User {
//!     id: i32,
//!     name: String,
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let pool = MySqlPool::connect("mysql://localhost/test").await?;
//! let email = "user@example.com";
//!
//! let mut query = PreparedQueryAs::<User, _>::new(
//!     "SELECT id, name FROM users WHERE email = :email",
//!     |q, key| match key {
//!         ":email" => q.bind(email),
//!         _ => q,
//!     }
//! )?;
//!
//! match query.fetch_optional(&pool).await? {
//!     Some(user) => println!("Found user: {}", user.name),
//!     None => println!("User not found"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## How It Works
//!
//! The library uses a three-step approach to avoid self-referential lifetime issues:
//!
//! 1. **Parse**: Extract named placeholders (`:name`) and convert SQL to use positional placeholders (`?`)
//! 2. **Store**: Keep the converted SQL, placeholder order, and binder function separately
//! 3. **Execute**: Construct a fresh SQLx `Query` on each execution with the correct lifetime
//!
//! This approach leverages HRTB (Higher-Rank Trait Bounds) to ensure the binder function
//! works with any lifetime, making the API both safe and flexible.
//!
//! ## Limitations
//!
//! - Currently only supports MySQL (PostgreSQL and SQLite support planned)
//! - Placeholder names must match `[a-zA-Z0-9_]+`
//! - All placeholders in the SQL must be handled by the binder function
//!
//! ## License
//!
//! Licensed under either of Apache License, Version 2.0 or MIT license at your option.

pub mod builder;
pub mod error;
pub mod query;
pub mod query_as;

pub use error::{Error, Result};
pub use query::PreparedQuery;
pub use query_as::PreparedQueryAs;

/// Convenience re-exports for common use cases
pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::PreparedQuery;
    pub use crate::PreparedQueryAs;
}
