# TypeSafe AI Rust SDK

Rust SDK for [TypeSafe AI](https://typesafe.ai).

## Quickstart

Add the dependency:

```toml
[dependencies]
typesafe = "0.1"
```

Set `TYPESAFE_API_KEY` in your environment, then create and use the client:

```rust
use std::collections::HashMap;

#[tokio::main]
async fn main() -> typesafe::Result<()> {
    let client = typesafe::Client::from_env()?; // reads TYPESAFE_API_KEY

    let resp = client.system_one(
        "I was charged twice. Please refund the duplicate charge today.",
        HashMap::from([
            ("category".into(), typesafe::Question::choice(
                "Categorize the message",
                HashMap::from([
                    ("billing".into(), "Billing issue".into()),
                    ("technical".into(), "Technical issue".into()),
                ]),
            )),
            ("urgency".into(), typesafe::Question::score(
                "Rate urgency",
                vec!["Can wait".into(), "Needs attention".into()],
            )),
            ("is_dupe".into(), typesafe::Question::noul(
                "Is this a duplicate charge?",
                Some(typesafe::NoulCriteria::new("Duplicate", "Not a duplicate")),
            )),
        ]),
        None,
    ).await?;

    println!("{}", resp.choices()["category"].choice); // "billing"
    println!("{}", resp.scores()["urgency"].score);    // 1.7
    println!("{}", resp.nouls()["is_dupe"].noul);      // 0.98

    Ok(())
}
```

Answer types are matched from your questions. The client supports builder configuration and per-call model overrides.

## Documentation

Learn what TypeSafe can do in the [TypeSafe docs](https://docs.typesafe.ai/).
See the SDK's [client](src/client.rs) and [types](src/question.rs) for API options and defaults.

## Configuration

Builder options: `.api_key()`, `.base_url()`, `.model()`, `.http_client()` on `Client::builder()`. Per-call model override with `SystemOneOptions::with_model()`. Non-2xx responses return `typesafe::Error::Api` with `status`, `detail`, and `body`.

| Environment Variable | Description | Default |
|---|---|---|
| `TYPESAFE_API_KEY` | API key (required unless set via builder) | — |
| `TYPESAFE_BASE_URL` | Base URL override | `https://api.typesafe.ai` |
| `TYPESAFE_DEFAULT_MODEL` | Default model name | `jev-latest` |

## Examples

Runnable examples live in [examples/](examples/):

```
TYPESAFE_API_KEY=... cargo run --example systemone
TYPESAFE_API_KEY=... cargo run --example listmodels
```

## License

MIT
