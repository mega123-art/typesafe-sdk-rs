use std::collections::HashMap;
use std::env;

use serde::{Deserialize, Serialize};

use crate::answer::Answer;
use crate::error::{Error, Result};
use crate::model::{Model, ModelMetadataList};
use crate::question::Question;

const DEFAULT_BASE_URL: &str = "https://api.typesafe.ai";
const DEFAULT_MODEL: &str = "jev-latest";

// ── Client ─────────────────────────────────────────────────────────────

/// A client for the TypeSafe AI API.
///
/// Create one via [`ClientBuilder`]:
///
/// ```no_run
/// # async fn run() -> typesafe::Result<()> {
/// let client = typesafe::Client::builder()
///     .api_key("sk-...")
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// Or use [`Client::from_env`] to read everything from environment variables:
///
/// ```no_run
/// # async fn run() -> typesafe::Result<()> {
/// let client = typesafe::Client::from_env()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    api_key: String,
    base_url: String,
    model: String,
    http: reqwest::Client,
}

/// Builder for constructing a [`Client`] with custom configuration.
///
/// Environment variables (`TYPESAFE_API_KEY`, `TYPESAFE_BASE_URL`,
/// `TYPESAFE_DEFAULT_MODEL`) are used as defaults; explicit builder
/// calls override them.
#[derive(Debug, Default)]
pub struct ClientBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
    http: Option<reqwest::Client>,
}

impl Client {
    /// Returns a new [`ClientBuilder`].
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Shorthand: build a client entirely from environment variables.
    ///
    /// Equivalent to `Client::builder().build()`.
    pub fn from_env() -> Result<Self> {
        ClientBuilder::default().build()
    }
}

impl ClientBuilder {
    /// Sets the API key, overriding `TYPESAFE_API_KEY`.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Sets the base URL, overriding `TYPESAFE_BASE_URL`.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Sets the default model, overriding `TYPESAFE_DEFAULT_MODEL`.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Sets the underlying [`reqwest::Client`].
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http = Some(client);
        self
    }

    /// Builds the [`Client`], reading environment variables as fallbacks.
    ///
    /// Returns [`Error::MissingApiKey`] if no API key is available.
    pub fn build(self) -> Result<Client> {
        let api_key = self
            .api_key
            .or_else(|| env::var("TYPESAFE_API_KEY").ok())
            .filter(|k| !k.is_empty())
            .ok_or(Error::MissingApiKey)?;

        let base_url = self
            .base_url
            .or_else(|| env::var("TYPESAFE_BASE_URL").ok().filter(|v| !v.is_empty()))
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        let model = self
            .model
            .or_else(|| {
                env::var("TYPESAFE_DEFAULT_MODEL")
                    .ok()
                    .filter(|v| !v.is_empty())
            })
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());

        let http = self.http.unwrap_or_default();

        Ok(Client {
            api_key,
            base_url,
            model,
            http,
        })
    }
}

// ── Token usage ────────────────────────────────────────────────────────

/// Token usage for the request.
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    /// Number of billable input tokens.
    pub input_tokens: u32,
    /// Number of output tokens (currently free of charge).
    pub output_tokens: u32,
}

// ── SystemOne ──────────────────────────────────────────────────────────

/// Request body for `POST /v1/systemone`.
#[derive(Debug, Serialize)]
struct SystemOneRequestBody {
    state: String,
    model: String,
    questions: HashMap<String, Question>,
}

/// The result of a [`Client::system_one`] call.
#[derive(Debug, Clone, Deserialize)]
pub struct SystemOneResponse {
    /// Name of the model that answered the questions.
    pub model: String,
    /// Token usage for this evaluation.
    pub usage: Usage,
    /// Answers keyed by question name.
    pub answers: HashMap<String, Answer>,
}

impl SystemOneResponse {
    /// Returns only the choice-typed answers.
    pub fn choices(&self) -> HashMap<&str, &crate::ChoiceAnswer> {
        self.answers
            .iter()
            .filter_map(|(k, a)| a.as_choice().map(|c| (k.as_str(), c)))
            .collect()
    }

    /// Returns only the score-typed answers.
    pub fn scores(&self) -> HashMap<&str, &crate::ScoreAnswer> {
        self.answers
            .iter()
            .filter_map(|(k, a)| a.as_score().map(|s| (k.as_str(), s)))
            .collect()
    }

    /// Returns only the noul-typed answers.
    pub fn nouls(&self) -> HashMap<&str, &crate::NoulAnswer> {
        self.answers
            .iter()
            .filter_map(|(k, a)| a.as_noul().map(|n| (k.as_str(), n)))
            .collect()
    }
}

/// Options for a single [`Client::system_one`] call.
#[derive(Debug, Default)]
pub struct SystemOneOptions {
    /// Override the client's default model for this call.
    pub model: Option<String>,
}

impl SystemOneOptions {
    /// Create options with a per-call model override.
    pub fn with_model(model: impl Into<String>) -> Self {
        SystemOneOptions {
            model: Some(model.into()),
        }
    }
}

impl Client {
    /// Ask questions about content via `POST /v1/systemone`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::collections::HashMap;
    /// # async fn run() -> typesafe::Result<()> {
    /// let client = typesafe::Client::from_env()?;
    /// let resp = client.system_one(
    ///     "I was charged twice.",
    ///     HashMap::from([
    ///         ("category".into(), typesafe::Question::choice(
    ///             "Categorize the message",
    ///             HashMap::from([
    ///                 ("billing".into(), "Billing issue".into()),
    ///                 ("technical".into(), "Technical issue".into()),
    ///             ]),
    ///         )),
    ///     ]),
    ///     None,
    /// ).await?;
    /// println!("{}", resp.choices()["category"].choice);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn system_one(
        &self,
        state: impl Into<String>,
        questions: HashMap<String, Question>,
        opts: Option<SystemOneOptions>,
    ) -> Result<SystemOneResponse> {
        let model = opts
            .and_then(|o| o.model)
            .unwrap_or_else(|| self.model.clone());

        let body = SystemOneRequestBody {
            state: state.into(),
            model,
            questions,
        };

        let url = format!("{}/v1/systemone", self.base_url);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        self.handle_response(resp).await
    }

    /// List available models via `GET /v1/models`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn run() -> typesafe::Result<()> {
    /// let client = typesafe::Client::from_env()?;
    /// let models = client.list_models().await?;
    /// for m in &models {
    ///     println!("{} — {}", m.name, m.description);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_models(&self) -> Result<Vec<Model>> {
        let url = format!("{}/v1/models", self.base_url);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await?;

        let list: ModelMetadataList = self.handle_response(resp).await?;
        Ok(list.models)
    }

    /// Shared response handler: checks status code, parses error detail on failure.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T> {
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let detail: Option<serde_json::Value> = serde_json::from_str(&body)
                .ok()
                .and_then(|v: serde_json::Value| v.get("detail").cloned());
            return Err(Error::Api {
                status: status.as_u16(),
                detail,
                body,
            });
        }
        let data: T = resp.json().await?;
        Ok(data)
    }
}
