//! Fictional seed for the prototype board. Dates are fixed around 2026-11-02.

use crate::domain::{
    Artifacts, BoardCard, EventKind, Outcome, Salary, Stage, Target, TimelineEvent, WorkModel,
};

#[allow(clippy::too_many_arguments)]
fn card(
    id: &str,
    title: &str,
    company: &str,
    source: &str,
    target: Target,
    work_model: WorkModel,
    score: u8,
    stage: Stage,
) -> BoardCard {
    BoardCard {
        id: id.into(),
        title: title.into(),
        company: company.into(),
        source: source.into(),
        target,
        work_model,
        location: None,
        score,
        stage,
        outcome: None,
        process_step: None,
        salary: None,
        artifacts: Artifacts { cv: false, coverage: false, interview_prep: false },
        last_activity: None,
        next_action: None,
        ai_running: None,
        unread: false,
        timeline: Vec::new(),
        next_action_date: None,
        recruiter_name: None,
        recruiter_email: None,
        recruiter_linkedin: None,
        job_url: None,
        application_url: None,
        notes: String::new(),
    }
}

fn ev(at: &str, kind: EventKind, label: &str) -> TimelineEvent {
    TimelineEvent { at: at.into(), kind, label: label.into() }
}

fn pay(pj: Option<u32>, clt: Option<u32>) -> Option<Salary> {
    Some(Salary { pj, clt })
}

fn text(value: &str) -> Option<String> {
    Some(value.into())
}

const ALL_ARTIFACTS: Artifacts = Artifacts { cv: true, coverage: true, interview_prep: true };

pub fn sample_cards() -> Vec<BoardCard> {
    use EventKind as E;
    use Stage as S;
    use Target as T;
    use WorkModel as W;

    vec![
        BoardCard {
            unread: true,
            last_activity: text("Nova vaga"),
            timeline: vec![ev("2026-11-02", E::Found, "Encontrada na Greenhouse")],
            ..card("c01", "Staff Engineer – LLM Platform", "Lumen Labs", "Greenhouse", T::AiLlm, W::Remote, 94, S::Inbox)
        },
        BoardCard {
            unread: true,
            last_activity: text("Nova vaga"),
            timeline: vec![ev("2026-11-02", E::Found, "Encontrada na Ashby")],
            ..card("c02", "Senior AI Engineer", "Cobalto AI", "Ashby", T::AiLlm, W::Remote, 88, S::Inbox)
        },
        BoardCard {
            last_activity: text("Adicionada ontem"),
            timeline: vec![ev("2026-11-01", E::Found, "Encontrada na Greenhouse")],
            ..card("c03", "Principal MLOps Engineer", "Synthetica", "Greenhouse", T::AiLlm, W::Remote, 90, S::Inbox)
        },
        BoardCard {
            location: text("São Paulo"),
            ai_running: text("claude"),
            timeline: vec![ev("2026-10-31", E::Found, "Encontrada na Gupy")],
            ..card("c04", "Tech Lead Full Stack", "Orbita Pay", "Gupy", T::TechLead, W::Hybrid, 82, S::Triage)
        },
        BoardCard {
            last_activity: text("Triada há 3 dias"),
            timeline: vec![ev("2026-10-30", E::Found, "Encontrada na Lever")],
            ..card("c05", "Engenheiro de Dados Sênior", "Nébula Data", "Lever", T::Data, W::Remote, 79, S::Triage)
        },
        BoardCard {
            last_activity: text("Triada há 2 dias"),
            timeline: vec![ev("2026-10-31", E::Found, "Encontrada na Workable")],
            ..card("c06", "Staff Backend Engineer", "Veloce Tech", "Workable", T::FullStack, W::Remote, 84, S::Triage)
        },
        BoardCard {
            salary: pay(Some(28_000), Some(21_000)),
            artifacts: Artifacts { cv: true, coverage: true, interview_prep: false },
            last_activity: text("Candidatura enviada há 3 dias"),
            timeline: vec![
                ev("2026-10-25", E::Found, "Alerta do Gmail"),
                ev("2026-10-30", E::Applied, "Candidatura enviada"),
            ],
            ..card("c07", "Lead AI Architect", "Vértice Saúde", "Gmail", T::AiLlm, W::Remote, 91, S::Applied)
        },
        BoardCard {
            salary: pay(Some(26_000), None),
            artifacts: Artifacts { cv: true, coverage: false, interview_prep: false },
            last_activity: text("Visualizado pelo RH há 1 dia"),
            timeline: vec![
                ev("2026-10-27", E::Found, "Encontrada no GitHub"),
                ev("2026-10-29", E::Applied, "Candidatura enviada"),
                ev("2026-11-01", E::Response, "Visualizado pelo RH"),
            ],
            ..card("c08", "Senior Backend Engineer", "Horizon Tech", "GitHub", T::FullStack, W::Remote, 85, S::Applied)
        },
        BoardCard {
            salary: pay(Some(32_000), None),
            artifacts: Artifacts { cv: true, coverage: true, interview_prep: false },
            last_activity: text("Candidatura enviada há 5 dias"),
            timeline: vec![
                ev("2026-10-26", E::Found, "Encontrada na Greenhouse"),
                ev("2026-10-28", E::Applied, "Candidatura enviada"),
            ],
            ..card("c09", "Tech Lead Distributed Systems", "CloudScale", "Greenhouse", T::TechLead, W::Remote, 87, S::Applied)
        },
        BoardCard {
            process_step: text("Técnica"),
            salary: pay(Some(34_000), Some(26_000)),
            artifacts: ALL_ARTIFACTS,
            next_action: text("Técnica amanhã 15:00"),
            timeline: vec![
                ev("2026-10-20", E::Applied, "Candidatura enviada"),
                ev("2026-10-23", E::Response, "RH respondeu pelo Gmail"),
                ev("2026-10-28", E::Interview, "Entrevista com RH"),
            ],
            ..card("c10", "Staff ML Engineer", "Quasar Bio", "Gmail", T::AiLlm, W::Remote, 92, S::InProcess)
        },
        BoardCard {
            location: text("São Paulo"),
            process_step: text("Case"),
            salary: pay(Some(30_000), None),
            artifacts: Artifacts { cv: true, coverage: true, interview_prep: false },
            next_action: text("Aguardando retorno do case"),
            timeline: vec![
                ev("2026-10-18", E::Applied, "Candidatura enviada"),
                ev("2026-10-22", E::Response, "Convite para o case"),
                ev("2026-10-27", E::Interview, "Case entregue"),
            ],
            ..card("c11", "Principal Systems Architect", "Nexus Infra", "Gupy", T::TechLead, W::Hybrid, 89, S::InProcess)
        },
        BoardCard {
            process_step: text("RH"),
            salary: pay(Some(29_000), Some(22_000)),
            artifacts: Artifacts { cv: true, coverage: false, interview_prep: true },
            next_action: text("Feedback em 2 dias"),
            timeline: vec![
                ev("2026-10-21", E::Applied, "Candidatura enviada"),
                ev("2026-10-24", E::Response, "RH respondeu"),
            ],
            ..card("c12", "Lead Platform Engineer", "KubeWorks", "Lever", T::FullStack, W::Remote, 86, S::InProcess)
        },
        BoardCard {
            process_step: text("Proposta"),
            salary: pay(Some(40_000), Some(31_000)),
            artifacts: ALL_ARTIFACTS,
            next_action: text("Proposta expira em 4 dias"),
            timeline: vec![
                ev("2026-10-10", E::Applied, "Candidatura enviada"),
                ev("2026-10-13", E::Response, "RH respondeu"),
                ev("2026-10-17", E::Interview, "Entrevista técnica"),
                ev("2026-10-31", E::Offer, "Carta proposta recebida"),
            ],
            ..card("c13", "Principal AI Lead", "Zenith Core", "Greenhouse", T::AiLlm, W::Remote, 96, S::Offer)
        },
        BoardCard {
            outcome: Some(Outcome::Hired),
            salary: pay(Some(42_000), Some(33_000)),
            artifacts: ALL_ARTIFACTS,
            last_activity: text("Contrato assinado"),
            timeline: vec![
                ev("2026-09-20", E::Applied, "Candidatura enviada"),
                ev("2026-09-24", E::Response, "RH respondeu"),
                ev("2026-10-08", E::Offer, "Proposta recebida"),
                ev("2026-10-12", E::Closed, "Contrato assinado"),
            ],
            ..card("c14", "Head of AI Architecture", "Atlas Cloud", "Ashby", T::AiLlm, W::Remote, 93, S::Closed)
        },
        BoardCard {
            location: text("São Paulo"),
            outcome: Some(Outcome::Rejected),
            salary: pay(Some(25_000), None),
            last_activity: text("Proposta recusada: abaixo do alvo"),
            timeline: vec![
                ev("2026-10-01", E::Applied, "Candidatura enviada"),
                ev("2026-10-06", E::Response, "RH respondeu"),
                ev("2026-10-15", E::Closed, "Proposta recusada"),
            ],
            ..card("c15", "Staff Data Engineer", "Solaris Fin", "Gupy", T::Data, W::Hybrid, 81, S::Closed)
        },
        BoardCard {
            outcome: Some(Outcome::NoResponse),
            salary: pay(None, Some(18_000)),
            last_activity: text("Sem resposta há 21 dias"),
            timeline: vec![
                ev("2026-10-05", E::Applied, "Candidatura enviada"),
                ev("2026-10-26", E::Closed, "Encerrada sem resposta"),
            ],
            ..card("c16", "Backend Tech Lead", "Prisma Retail", "Gupy", T::TechLead, W::Onsite, 72, S::Closed)
        },
        BoardCard {
            outcome: Some(Outcome::Discarded),
            last_activity: text("Descartada na triagem"),
            timeline: vec![
                ev("2026-10-20", E::Found, "Encontrada na ProgramaThor"),
                ev("2026-10-21", E::Closed, "Descartada na triagem"),
            ],
            ..card("c17", "Desenvolvedor Full Stack Sênior", "Delta Commerce", "ProgramaThor", T::FullStack, W::Remote, 70, S::Closed)
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn seeds_every_board_stage() {
        let cards = sample_cards();
        for stage in Stage::ALL {
            assert!(cards.iter().any(|c| c.stage == stage), "no card in {stage:?}");
        }
    }

    #[test]
    fn closed_cards_carry_an_outcome() {
        for card in sample_cards() {
            assert_eq!(card.stage == Stage::Closed, card.outcome.is_some(), "{}", card.id);
        }
    }

    #[test]
    fn cards_before_applying_have_no_salary() {
        for card in sample_cards() {
            if matches!(card.stage, Stage::Inbox | Stage::Triage) {
                assert!(card.salary.is_none(), "{}", card.id);
            }
        }
    }

    #[test]
    fn ids_are_unique() {
        let cards = sample_cards();
        let ids: HashSet<_> = cards.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids.len(), cards.len());
    }
}
