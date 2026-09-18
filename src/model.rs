use serde::Deserialize;

/// A model or model alias available to the authenticated account.
#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    /// Model name or alias accepted by the request's `model` field.
    pub name: String,
    /// Human-readable description of the model and its capabilities.
    pub description: String,
    /// Model release date, formatted as `YYYY-MM-DD`.
    pub release_date: String,
}

/// Wrapper for the `GET /v1/models` response body.
#[derive(Debug, Deserialize)]
pub(crate) struct ModelMetadataList {
    pub models: Vec<Model>,
}
