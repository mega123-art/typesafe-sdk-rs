//! Asks typed questions about a support message using SystemOne.
//!
//! Run with:
//! ```text
//! TYPESAFE_API_KEY=... cargo run --example systemone
//! ```

use std::collections::HashMap;

#[tokio::main]
async fn main() -> typesafe::Result<()> {
    let client = typesafe::Client::from_env()?; // reads TYPESAFE_API_KEY

    let resp = client
        .system_one(
            "I was charged twice. Please refund the duplicate charge today.",
            HashMap::from([
                (
                    "category".into(),
                    typesafe::Question::choice(
                        "Categorize the message",
                        HashMap::from([
                            ("billing".into(), "Billing issue".into()),
                            ("technical".into(), "Technical issue".into()),
                        ]),
                    ),
                ),
                (
                    "urgency".into(),
                    typesafe::Question::score(
                        "Rate urgency",
                        vec!["Can wait".into(), "Needs attention".into()],
                    ),
                ),
                (
                    "is_dupe".into(),
                    typesafe::Question::noul(
                        "Is this a duplicate charge?",
                        Some(typesafe::NoulCriteria::new("Duplicate", "Not a duplicate")),
                    ),
                ),
            ]),
            None,
        )
        .await?;

    let cat = &resp.choices()["category"];
    println!(
        "category: {} (confidence {:.2})",
        cat.choice, cat.confidence
    );
    println!("urgency:  {:.2}", resp.scores()["urgency"].score);
    println!("is_dupe:  {:.2}", resp.nouls()["is_dupe"].noul);
    println!(
        "usage:    {} in / {} out",
        resp.usage.input_tokens, resp.usage.output_tokens
    );

    Ok(())
}
