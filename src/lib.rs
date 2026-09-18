//! # typesafe
//!
//! Rust SDK for the [TypeSafe AI API](https://api.typesafe.ai).
//!
//! ## Quick start
//!
//! ```no_run
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> typesafe::Result<()> {
//!     let client = typesafe::Client::from_env()?; // reads TYPESAFE_API_KEY
//!
//!     let resp = client.system_one(
//!         "I was charged twice. Please refund the duplicate charge today.",
//!         HashMap::from([
//!             ("category".into(), typesafe::Question::choice(
//!                 "Categorize the message",
//!                 HashMap::from([
//!                     ("billing".into(), "Billing issue".into()),
//!                     ("technical".into(), "Technical issue".into()),
//!                 ]),
//!             )),
//!             ("urgency".into(), typesafe::Question::score(
//!                 "Rate urgency",
//!                 vec!["Can wait".into(), "Needs attention".into()],
//!             )),
//!             ("is_dupe".into(), typesafe::Question::noul(
//!                 "Is this a duplicate charge?",
//!                 Some(typesafe::NoulCriteria::new("Duplicate", "Not a duplicate")),
//!             )),
//!         ]),
//!         None,
//!     ).await?;
//!
//!     println!("{}", resp.choices()["category"].choice);   // "billing"
//!     println!("{}", resp.scores()["urgency"].score);      // 1.7
//!     println!("{}", resp.nouls()["is_dupe"].noul);        // 0.98
//!
//!     let models = client.list_models().await?;
//!     for m in &models {
//!         println!("{} — {}", m.name, m.description);
//!     }
//!
//!     Ok(())
//! }
//! ```

mod answer;
mod client;
mod error;
mod model;
mod question;

// Re-export public API
pub use answer::{Answer, ChoiceAnswer, NoulAnswer, ScoreAnswer};
pub use client::{Client, ClientBuilder, SystemOneOptions, SystemOneResponse, Usage};
pub use error::{Error, Result};
pub use model::Model;
pub use question::{NoulCriteria, Question};
