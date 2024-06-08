//! src/routes/mod.rs
mod health_check;
mod subscriptions;
mod subscriptions_confirm;

mod newletters;
pub use health_check::*;
pub use newletters::*;
pub use subscriptions::*;
pub use subscriptions_confirm::*;
