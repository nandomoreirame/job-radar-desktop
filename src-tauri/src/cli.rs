use crate::import::ImportReport;
use std::ffi::OsString;
use std::path::PathBuf;
pub const USAGE: &str =
    "usage: job-radar-desktop import --seen <seen.db> [--candidates <candidates.json>]";
#[derive(Debug, PartialEq, Eq)]
pub struct ImportArgs {
    pub seen: PathBuf,
    pub candidates: Option<PathBuf>,
}
pub fn parse(args: &[OsString]) -> Result<Option<ImportArgs>, String> {
    let Some((command, rest)) = args.split_first() else {
        return Ok(None);
    };
    if command != "import" {
        return Err(USAGE.into());
    }
    let mut seen = None;
    let mut candidates = None;
    let mut it = rest.iter();
    while let Some(flag) = it.next() {
        let value = it.next().ok_or(USAGE)?;
        match flag.to_str() {
            Some("--seen") => seen = Some(PathBuf::from(value)),
            Some("--candidates") => candidates = Some(PathBuf::from(value)),
            _ => return Err(USAGE.into()),
        }
    }
    Ok(Some(ImportArgs {
        seen: seen.ok_or(USAGE)?,
        candidates,
    }))
}
pub fn format_report(r: &ImportReport) -> String {
    format!(
        "inbox: {}\ndiscarded: {}\nduplicates: {}\ndismissed: {}\ninvalid: {}",
        r.inbox, r.discarded, r.duplicates, r.dismissed, r.invalid
    )
}
#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_the_import_command() {
        assert_eq!(parse(&args(&[])), Ok(None));
        assert_eq!(
            parse(&args(&[
                "import",
                "--seen",
                "a.db",
                "--candidates",
                "b.json"
            ])),
            Ok(Some(ImportArgs {
                seen: "a.db".into(),
                candidates: Some("b.json".into())
            }))
        );
        assert_eq!(
            parse(&args(&["import", "--seen", "a.db"])),
            Ok(Some(ImportArgs {
                seen: "a.db".into(),
                candidates: None
            }))
        );
        for bad in [
            &["import"][..],
            &["import", "--seen"],
            &["import", "--nope", "x"],
            &["other"],
        ] {
            assert!(parse(&args(bad)).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn reports_every_count() {
        let report = ImportReport {
            inbox: 1,
            discarded: 2,
            duplicates: 3,
            dismissed: 4,
            invalid: 5,
        };
        assert_eq!(
            format_report(&report),
            "inbox: 1\ndiscarded: 2\nduplicates: 3\ndismissed: 4\ninvalid: 5"
        );
    }
}
