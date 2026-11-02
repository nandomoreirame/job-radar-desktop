//! Validation for the card fields the user edits. Everything from the webview
//! is untrusted: text is trimmed and bounded, links must be http(s) (no
//! `javascript:`), dates must exist. Errors read `<field>: <reason>` with the
//! camelCase field name, so the form can point at the offending input.

use crate::domain::{CardDetails, Salary};

pub const MAX_TEXT_LEN: usize = 200;
pub const MAX_URL_LEN: usize = 500;
pub const MAX_NOTES_LEN: usize = 5_000;
/// Monthly BRL; anything above is a typo, not a salary.
pub const MAX_SALARY: u32 = 1_000_000;

fn fail(field: &str, reason: &str) -> String {
    format!("{field}: {reason}")
}

fn text(field: &str, value: Option<String>, max: usize) -> Result<Option<String>, String> {
    let Some(value) = value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    if value.chars().count() > max {
        return Err(fail(field, "too_long"));
    }
    Ok(Some(value))
}

fn email(field: &str, value: Option<String>) -> Result<Option<String>, String> {
    let value = text(field, value, MAX_TEXT_LEN)?;
    if let Some(v) = &value {
        let valid = match v.split_once('@') {
            Some((local, domain)) => {
                !local.is_empty()
                    && !domain.contains('@')
                    && domain.contains('.')
                    && !domain.starts_with('.')
                    && !domain.ends_with('.')
                    && !v.chars().any(char::is_whitespace)
            }
            None => false,
        };
        if !valid {
            return Err(fail(field, "invalid_email"));
        }
    }
    Ok(value)
}

fn url(field: &str, value: Option<String>) -> Result<Option<String>, String> {
    let value = text(field, value, MAX_URL_LEN)?;
    if let Some(v) = &value {
        let rest = v
            .strip_prefix("https://")
            .or_else(|| v.strip_prefix("http://"));
        let host = rest.map(|r| r.split(['/', '?', '#']).next().unwrap_or(""));
        let valid = matches!(host, Some(h) if !h.is_empty()) && !v.chars().any(char::is_whitespace);
        if !valid {
            return Err(fail(field, "invalid_url"));
        }
    }
    Ok(value)
}

fn date(field: &str, value: Option<String>) -> Result<Option<String>, String> {
    let value = text(field, value, MAX_TEXT_LEN)?;
    if value.as_deref().is_some_and(|v| !is_iso_date(v)) {
        return Err(fail(field, "invalid_date"));
    }
    Ok(value)
}

fn is_iso_date(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    let [y, m, d] = parts.as_slice() else {
        return false;
    };
    if y.len() != 4 || m.len() != 2 || d.len() != 2 {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (y.parse::<u32>(), m.parse::<u32>(), d.parse::<u32>())
    else {
        return false;
    };
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}

fn salary(value: Option<Salary>) -> Result<Option<Salary>, String> {
    let Some(s) = value else { return Ok(None) };
    if s.pj.is_some_and(|v| v > MAX_SALARY) {
        return Err(fail("salaryPj", "out_of_range"));
    }
    if s.clt.is_some_and(|v| v > MAX_SALARY) {
        return Err(fail("salaryClt", "out_of_range"));
    }
    // Both regimes empty means nothing was negotiated.
    Ok(if s.pj.is_none() && s.clt.is_none() {
        None
    } else {
        Some(s)
    })
}

pub fn normalize_details(input: CardDetails) -> Result<CardDetails, String> {
    let notes = input.notes.trim().to_owned();
    if notes.chars().count() > MAX_NOTES_LEN {
        return Err(fail("notes", "too_long"));
    }
    Ok(CardDetails {
        salary: salary(input.salary)?,
        process_step: text("processStep", input.process_step, MAX_TEXT_LEN)?,
        next_action: text("nextAction", input.next_action, MAX_TEXT_LEN)?,
        next_action_date: date("nextActionDate", input.next_action_date)?,
        recruiter_name: text("recruiterName", input.recruiter_name, MAX_TEXT_LEN)?,
        recruiter_email: email("recruiterEmail", input.recruiter_email)?,
        recruiter_linkedin: url("recruiterLinkedin", input.recruiter_linkedin)?,
        job_url: url("jobUrl", input.job_url)?,
        application_url: url("applicationUrl", input.application_url)?,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CardDetails, Salary};

    fn details() -> CardDetails {
        CardDetails {
            salary: None,
            process_step: None,
            next_action: None,
            next_action_date: None,
            recruiter_name: None,
            recruiter_email: None,
            recruiter_linkedin: None,
            job_url: None,
            application_url: None,
            notes: String::new(),
        }
    }

    #[test]
    fn accepts_full_valid_details() {
        let input = CardDetails {
            salary: Some(Salary {
                pj: Some(28_000),
                clt: Some(21_000),
            }),
            process_step: Some("Técnica".into()),
            next_action: Some("Entrevista técnica".into()),
            next_action_date: Some("2026-11-05".into()),
            recruiter_name: Some("Ana Souza".into()),
            recruiter_email: Some("ana@example.com".into()),
            recruiter_linkedin: Some("https://www.linkedin.com/in/ana".into()),
            job_url: Some("https://jobs.example.com/123".into()),
            application_url: Some("http://portal.example.com/app/9".into()),
            notes: "Falar do projeto de ETL.".into(),
        };
        assert_eq!(normalize_details(input.clone()), Ok(input));
    }

    #[test]
    fn trims_text_and_drops_empty_fields() {
        let input = CardDetails {
            recruiter_name: Some("  Ana  ".into()),
            process_step: Some("   ".into()),
            notes: "  nota  ".into(),
            salary: Some(Salary {
                pj: None,
                clt: None,
            }),
            ..details()
        };
        let out = normalize_details(input).unwrap();
        assert_eq!(out.recruiter_name.as_deref(), Some("Ana"));
        assert_eq!(out.process_step, None);
        assert_eq!(out.notes, "nota");
        assert_eq!(out.salary, None);
    }

    #[test]
    fn rejects_an_invalid_email() {
        for email in [
            "ana",
            "ana@",
            "@example.com",
            "ana @example.com",
            "ana@example",
        ] {
            let input = CardDetails {
                recruiter_email: Some(email.into()),
                ..details()
            };
            assert_eq!(
                normalize_details(input),
                Err("recruiterEmail: invalid_email".into()),
                "{email}"
            );
        }
    }

    #[test]
    fn rejects_urls_that_are_not_http() {
        for url in [
            "javascript:alert(1)",
            "ftp://example.com",
            "https://",
            "example.com",
            "https://exa mple.com",
        ] {
            let input = CardDetails {
                job_url: Some(url.into()),
                ..details()
            };
            assert_eq!(
                normalize_details(input),
                Err("jobUrl: invalid_url".into()),
                "{url}"
            );
        }
    }

    #[test]
    fn rejects_impossible_dates() {
        for date in [
            "2026-02-30",
            "2026-13-01",
            "26-11-05",
            "2026-11-5",
            "amanhã",
        ] {
            let input = CardDetails {
                next_action_date: Some(date.into()),
                ..details()
            };
            assert_eq!(
                normalize_details(input),
                Err("nextActionDate: invalid_date".into()),
                "{date}"
            );
        }
        let leap = CardDetails {
            next_action_date: Some("2028-02-29".into()),
            ..details()
        };
        assert!(normalize_details(leap).is_ok());
    }

    #[test]
    fn rejects_oversized_text() {
        let notes = CardDetails {
            notes: "x".repeat(MAX_NOTES_LEN + 1),
            ..details()
        };
        assert_eq!(normalize_details(notes), Err("notes: too_long".into()));
        let name = CardDetails {
            recruiter_name: Some("x".repeat(MAX_TEXT_LEN + 1)),
            ..details()
        };
        assert_eq!(
            normalize_details(name),
            Err("recruiterName: too_long".into())
        );
    }

    #[test]
    fn rejects_a_salary_out_of_range() {
        let input = CardDetails {
            salary: Some(Salary {
                pj: Some(MAX_SALARY + 1),
                clt: None,
            }),
            ..details()
        };
        assert_eq!(
            normalize_details(input),
            Err("salaryPj: out_of_range".into())
        );
    }
}
