use serde::Serialize;
use std::collections::HashMap;

/// A question to ask about the content supplied in `state`.
///
/// Three question types are supported:
/// - [`Question::Choice`] picks one option from named criteria.
/// - [`Question::Score`] rates on a scale described by ordered criteria.
/// - [`Question::Noul`] answers a true/false question with a probability.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
    /// Picks one option from named criteria.
    Choice {
        /// What the model should decide when choosing an option.
        #[serde(skip_serializing_if = "Option::is_none")]
        instructions: Option<String>,
        /// Choice names and descriptions of when each applies.
        criteria: HashMap<String, String>,
    },

    /// Rates on a scale described by ordered criteria.
    Score {
        /// What the model should rate.
        #[serde(skip_serializing_if = "Option::is_none")]
        instructions: Option<String>,
        /// Ordered descriptions of the score levels (index = score).
        criteria: Vec<String>,
    },

    /// Answers a yes/no question with a probability.
    Noul {
        /// The yes/no question or statement to evaluate.
        #[serde(skip_serializing_if = "Option::is_none")]
        instructions: Option<String>,
        /// Criteria clarifying what counts as a yes or no answer.
        #[serde(skip_serializing_if = "Option::is_none")]
        criteria: Option<NoulCriteria>,
    },
}

/// Criteria defining what counts as a yes or no answer.
#[derive(Debug, Clone, Serialize)]
pub struct NoulCriteria {
    /// What counts as a yes answer.
    #[serde(rename = "true")]
    pub r#true: String,
    /// What counts as a no answer.
    #[serde(rename = "false")]
    pub r#false: String,
}

// ── Convenience constructors ───────────────────────────────────────────

impl Question {
    /// Create a choice question.
    pub fn choice(
        instructions: impl Into<String>,
        criteria: HashMap<String, String>,
    ) -> Self {
        Question::Choice {
            instructions: Some(instructions.into()),
            criteria,
        }
    }

    /// Create a score question.
    pub fn score(instructions: impl Into<String>, criteria: Vec<String>) -> Self {
        Question::Score {
            instructions: Some(instructions.into()),
            criteria,
        }
    }

    /// Create a noul (yes/no) question.
    pub fn noul(instructions: impl Into<String>, criteria: Option<NoulCriteria>) -> Self {
        Question::Noul {
            instructions: Some(instructions.into()),
            criteria,
        }
    }
}

impl NoulCriteria {
    /// Create noul criteria from true/false descriptions.
    pub fn new(r#true: impl Into<String>, r#false: impl Into<String>) -> Self {
        NoulCriteria {
            r#true: r#true.into(),
            r#false: r#false.into(),
        }
    }
}
