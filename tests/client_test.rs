use std::collections::HashMap;

use wiremock::matchers::{bearer_token, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SYSTEM_ONE_FIXTURE: &str = r#"{
  "model": "jev-latest",
  "usage": {"input_tokens": 120, "output_tokens": 12},
  "answers": {
    "category": {"type": "choice", "choice": "billing", "confidence": 0.9, "probabilities": {"billing": 0.8, "technical": 0.1}},
    "urgency":  {"type": "score", "score": 1.7, "confidence": 0.9, "legend": {"0": "Can wait"}, "probabilities": {"0": 0.1, "1": 0.9}},
    "is_dupe":  {"type": "noul", "noul": 0.98}
  }
}"#;

fn build_client(base_url: &str) -> typesafe::Client {
    typesafe::Client::builder()
        .api_key("test-key")
        .base_url(base_url)
        .build()
        .expect("client should build")
}

fn sample_questions() -> HashMap<String, typesafe::Question> {
    HashMap::from([
        (
            "category".into(),
            typesafe::Question::choice(
                "categorize",
                HashMap::from([
                    ("billing".into(), "b".into()),
                    ("technical".into(), "t".into()),
                ]),
            ),
        ),
        (
            "urgency".into(),
            typesafe::Question::score(
                "rate",
                vec!["Can wait".into(), "Needs attention".into()],
            ),
        ),
        (
            "is_dupe".into(),
            typesafe::Question::noul(
                "dupe?",
                Some(typesafe::NoulCriteria::new("y", "n")),
            ),
        ),
    ])
}

#[tokio::test]
async fn test_system_one() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/systemone"))
        .and(bearer_token("test-key"))
        .and(header("content-type", "application/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(SYSTEM_ONE_FIXTURE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = build_client(&server.uri());

    let resp = client
        .system_one("I was charged twice.", sample_questions(), None)
        .await
        .expect("system_one should succeed");

    // Verify response parsing
    assert_eq!(resp.model, "jev-latest");
    assert_eq!(resp.usage.input_tokens, 120);
    assert_eq!(resp.usage.output_tokens, 12);

    // Choice answer
    let choices = resp.choices();
    let cat = choices.get("category").expect("should have category");
    assert_eq!(cat.choice, "billing");
    assert!((cat.confidence - 0.9).abs() < f64::EPSILON);
    assert!((cat.probabilities["billing"] - 0.8).abs() < f64::EPSILON);

    // Score answer
    let scores = resp.scores();
    let urg = scores.get("urgency").expect("should have urgency");
    assert!((urg.score - 1.7).abs() < f64::EPSILON);
    assert_eq!(urg.legend.get("0").map(|s| s.as_str()), Some("Can wait"));

    // Noul answer
    let nouls = resp.nouls();
    let dupe = nouls.get("is_dupe").expect("should have is_dupe");
    assert!((dupe.noul - 0.98).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_system_one_model_override() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/systemone"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"model":"jev-2","usage":{"input_tokens":0,"output_tokens":0},"answers":{}}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let client = build_client(&server.uri());
    let resp = client
        .system_one(
            "s",
            HashMap::new(),
            Some(typesafe::SystemOneOptions::with_model("jev-2")),
        )
        .await
        .expect("should succeed");

    assert_eq!(resp.model, "jev-2");
}

#[tokio::test]
async fn test_list_models() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(bearer_token("test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"models":[{"name":"jev-latest","description":"d","release_date":"2026-01-01"}]}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let client = build_client(&server.uri());
    let models = client.list_models().await.expect("should succeed");

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].name, "jev-latest");
    assert_eq!(models[0].description, "d");
    assert_eq!(models[0].release_date, "2026-01-01");
}

#[tokio::test]
async fn test_validation_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/systemone"))
        .respond_with(ResponseTemplate::new(422).set_body_raw(
            r#"{"detail":[{"loc":["body","state"],"msg":"field required","type":"missing"}]}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let client = build_client(&server.uri());
    let err = client
        .system_one("", HashMap::new(), None)
        .await
        .expect_err("should fail with 422");

    match &err {
        typesafe::Error::Api {
            status,
            detail,
            body: _,
        } => {
            assert_eq!(*status, 422);
            assert!(detail.is_some(), "should have detail");
        }
        other => panic!("expected Error::Api, got: {:?}", other),
    }

    // Error display should mention HTTP 422
    let msg = err.to_string();
    assert!(msg.contains("422"), "error message should contain 422: {msg}");
}

#[tokio::test]
async fn test_missing_api_key() {
    // Temporarily unset the env var for this test
    let saved = std::env::var("TYPESAFE_API_KEY").ok();
    std::env::remove_var("TYPESAFE_API_KEY");

    let result = typesafe::Client::builder().build();

    // Restore env var if it was set
    if let Some(v) = saved {
        std::env::set_var("TYPESAFE_API_KEY", v);
    }

    match result {
        Err(typesafe::Error::MissingApiKey) => {} // expected
        other => panic!("expected MissingApiKey, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_question_serialization() {
    // Verify that questions serialize with the correct `type` discriminator
    let q = typesafe::Question::choice(
        "Categorize",
        HashMap::from([("a".into(), "A".into())]),
    );
    let json = serde_json::to_value(&q).expect("should serialize");
    assert_eq!(json["type"], "choice");
    assert_eq!(json["instructions"], "Categorize");
    assert_eq!(json["criteria"]["a"], "A");

    let q = typesafe::Question::score("Rate", vec!["Low".into(), "High".into()]);
    let json = serde_json::to_value(&q).expect("should serialize");
    assert_eq!(json["type"], "score");
    assert_eq!(json["criteria"][0], "Low");

    let q = typesafe::Question::noul(
        "Is spam?",
        Some(typesafe::NoulCriteria::new("Spam", "Not spam")),
    );
    let json = serde_json::to_value(&q).expect("should serialize");
    assert_eq!(json["type"], "noul");
    assert_eq!(json["criteria"]["true"], "Spam");
    assert_eq!(json["criteria"]["false"], "Not spam");
}
