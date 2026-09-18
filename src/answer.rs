use serde::Deserialize;
use std::collections::HashMap;

/// An answer whose type matches the corresponding question.
///
/// Deserialized from the `answers` map in a [`SystemOneResponse`](crate::SystemOneResponse).
/// The `type` field in the JSON determines which variant is used.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
    /// Answer to a choice question.
    Choice(ChoiceAnswer),
    /// Answer to a score question.
    Score(ScoreAnswer),
    /// Answer to a noul question.
    Noul(NoulAnswer),
}

/// The selected choice, confidence, and probabilities for a choice question.
#[derive(Debug, Clone, Deserialize)]
pub struct ChoiceAnswer {
    /// The name of the choice with the highest probability.
    pub choice: String,
    /// Confidence in the selected choice, from 0 to 1.
    pub confidence: f64,
    /// Probability of each choice, keyed by choice name.
    pub probabilities: HashMap<String, f64>,
}

/// An expected score with its rubric, confidence, and score-level probabilities.
#[derive(Debug, Clone, Deserialize)]
pub struct ScoreAnswer {
    /// Probability-weighted average of the rubric levels.
    pub score: f64,
    /// Confidence in the score, from 0 to 1.
    pub confidence: f64,
    /// The requested criteria mapped to their score levels.
    pub legend: HashMap<String, String>,
    /// Probability of each score level.
    pub probabilities: HashMap<String, f64>,
}

/// The probability of a yes answer or a true statement.
#[derive(Debug, Clone, Deserialize)]
pub struct NoulAnswer {
    /// Probability of yes/true, from 0 to 1.
    pub noul: f64,
}

// ── Helper methods on Answer ───────────────────────────────────────────

impl Answer {
    /// Returns the inner [`ChoiceAnswer`] if this is a `Choice` variant.
    pub fn as_choice(&self) -> Option<&ChoiceAnswer> {
        match self {
            Answer::Choice(a) => Some(a),
            _ => None,
        }
    }

    /// Returns the inner [`ScoreAnswer`] if this is a `Score` variant.
    pub fn as_score(&self) -> Option<&ScoreAnswer> {
        match self {
            Answer::Score(a) => Some(a),
            _ => None,
        }
    }

    /// Returns the inner [`NoulAnswer`] if this is a `Noul` variant.
    pub fn as_noul(&self) -> Option<&NoulAnswer> {
        match self {
            Answer::Noul(a) => Some(a),
            _ => None,
        }
    }
}
