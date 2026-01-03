//! Racoon Database Service
//!
//! Valkey-based state database with schema definitions

pub mod schema;

pub use schema::{Database, DbError, DbResult};
