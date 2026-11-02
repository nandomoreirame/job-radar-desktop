//! Board domain shared with the frontend. `cargo test` exports these types to
//! `src/shared/bindings` through ts-rs, so the React side never redeclares them.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Stage {
    Inbox,
    Triage,
    Applied,
    InProcess,
    Offer,
    Closed,
}

impl Stage {
    /// Column order on the board, left to right.
    pub const ALL: [Stage; 6] = [
        Stage::Inbox,
        Stage::Triage,
        Stage::Applied,
        Stage::InProcess,
        Stage::Offer,
        Stage::Closed,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Outcome {
    Hired,
    Rejected,
    Discarded,
    NoResponse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Target {
    AiLlm,
    FullStack,
    TechLead,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum WorkModel {
    Remote,
    Hybrid,
    Onsite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum EventKind {
    Found,
    Applied,
    Response,
    Interview,
    Offer,
    Closed,
}

/// Negotiated monthly pay in BRL. Either regime may be missing; both missing
/// means nothing was negotiated yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Salary {
    pub pj: Option<u32>,
    pub clt: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Artifacts {
    pub cv: bool,
    pub coverage: bool,
    pub interview_prep: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimelineEvent {
    /// ISO date, `YYYY-MM-DD`.
    pub at: String,
    pub kind: EventKind,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BoardCard {
    pub id: String,
    pub title: String,
    pub company: String,
    pub source: String,
    pub target: Option<Target>,
    pub work_model: Option<WorkModel>,
    pub location: Option<String>,
    pub score: Option<u8>,
    pub stage: Stage,
    /// Set only when `stage` is `Closed`.
    pub outcome: Option<Outcome>,
    /// Free label for the current interview step ("Técnica", "Case").
    pub process_step: Option<String>,
    pub salary: Option<Salary>,
    pub artifacts: Artifacts,
    pub last_activity: Option<String>,
    pub next_action: Option<String>,
    /// Name of the AI CLI working on this card, if any.
    pub ai_running: Option<String>,
    pub unread: bool,
    pub timeline: Vec<TimelineEvent>,
    /// Date of `next_action`, ISO `YYYY-MM-DD`.
    pub next_action_date: Option<String>,
    pub recruiter_name: Option<String>,
    pub recruiter_email: Option<String>,
    pub recruiter_linkedin: Option<String>,
    pub job_url: Option<String>,
    pub application_url: Option<String>,
    /// The user's own notes about this application.
    pub notes: String,
}

/// The fields of a card the user edits (the job itself comes from the crawler).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CardDetails {
    pub salary: Option<Salary>,
    pub process_step: Option<String>,
    pub next_action: Option<String>,
    pub next_action_date: Option<String>,
    pub recruiter_name: Option<String>,
    pub recruiter_email: Option<String>,
    pub recruiter_linkedin: Option<String>,
    pub job_url: Option<String>,
    pub application_url: Option<String>,
    pub notes: String,
}

impl BoardCard {
    pub fn apply(&mut self, details: CardDetails) {
        self.salary = details.salary;
        self.process_step = details.process_step;
        self.next_action = details.next_action;
        self.next_action_date = details.next_action_date;
        self.recruiter_name = details.recruiter_name;
        self.recruiter_email = details.recruiter_email;
        self.recruiter_linkedin = details.recruiter_linkedin;
        self.job_url = details.job_url;
        self.application_url = details.application_url;
        self.notes = details.notes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn stage_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_value(Stage::InProcess).unwrap(),
            json!("in_process")
        );
    }

    #[test]
    fn outcome_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_value(Outcome::NoResponse).unwrap(),
            json!("no_response")
        );
    }

    #[test]
    fn salary_keeps_a_missing_regime_as_null() {
        let salary = Salary {
            pj: Some(28_000),
            clt: None,
        };
        assert_eq!(
            serde_json::to_value(salary).unwrap(),
            json!({ "pj": 28000, "clt": null })
        );
    }

    #[test]
    fn artifacts_use_camel_case() {
        let artifacts = Artifacts {
            cv: true,
            coverage: false,
            interview_prep: true,
        };
        assert_eq!(
            serde_json::to_value(artifacts).unwrap(),
            json!({ "cv": true, "coverage": false, "interviewPrep": true })
        );
    }

    #[test]
    fn stage_order_matches_the_board() {
        assert_eq!(Stage::ALL.first(), Some(&Stage::Inbox));
        assert_eq!(Stage::ALL.last(), Some(&Stage::Closed));
        assert_eq!(Stage::ALL.len(), 6);
    }
}
