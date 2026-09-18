# API

Full reference: [docs.rs](https://docs.rs/typesafe)

The upstream contract is in [openapi.json](openapi.json), fetched from `https://api.typesafe.ai/openapi.json`. The SDK is written by hand against this spec, with each Rust type matching an upstream schema:

| OpenAPI schema | Rust type |
|---|---|
| `SystemOneRequest` | `Client::system_one` parameters |
| `ChoiceQuestion`, `ScoreQuestion`, `NoulQuestion` | `Question::Choice`, `Question::Score`, `Question::Noul` |
| `SystemOneResponse`, `Usage` | `SystemOneResponse`, `Usage` |
| `ChoiceAnswer`, `ScoreAnswer`, `NoulAnswer` | `ChoiceAnswer`, `ScoreAnswer`, `NoulAnswer` |
| `ModelMetadata`, `ModelMetadataList` | `Model`, `Client::list_models` result |
| `HTTPValidationError` | `Error::Api { detail, .. }` |

## Client

- `typesafe::Client::builder() -> ClientBuilder`
- `typesafe::Client::from_env() -> Result<Client>`

Builder methods:

- `.api_key(key: impl Into<String>)`
- `.base_url(url: impl Into<String>)`
- `.model(model: impl Into<String>)`
- `.http_client(client: reqwest::Client)`
- `.build() -> Result<Client>`

Environment variables: `TYPESAFE_API_KEY`, `TYPESAFE_BASE_URL`, `TYPESAFE_DEFAULT_MODEL`.

## SystemOne

`POST /v1/systemone`

- `client.system_one(state, questions, opts).await -> Result<SystemOneResponse>`
- `SystemOneOptions::with_model(model)` — per-call model override

Question types (variants of `Question` enum):

- `Question::choice(instructions, criteria: HashMap<String, String>)`
- `Question::score(instructions, criteria: Vec<String>)`
- `Question::noul(instructions, criteria: Option<NoulCriteria>)`

Response:

- `resp.model: String`
- `resp.usage: Usage` — `input_tokens`, `output_tokens`
- `resp.answers: HashMap<String, Answer>` — union of the answer types below
- `resp.choices() -> HashMap<&str, &ChoiceAnswer>` — `choice`, `confidence`, `probabilities`
- `resp.scores() -> HashMap<&str, &ScoreAnswer>` — `score`, `confidence`, `legend`, `probabilities`
- `resp.nouls() -> HashMap<&str, &NoulAnswer>` — `noul`

## Models

`GET /v1/models`

- `client.list_models().await -> Result<Vec<Model>>` — `name`, `description`, `release_date`

## Errors

The `typesafe::Error` enum:

- `Error::MissingApiKey` — no API key configured
- `Error::Api { status: u16, detail: Option<Value>, body: String }` — non-2xx response
- `Error::Http(reqwest::Error)` — transport error
