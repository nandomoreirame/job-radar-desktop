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
    fn args(v: &[&str]) -> Vec<OsString> {
        v.iter().map(OsString::from).collect()
    }
    #[test]
    fn parses() {
        assert_eq!(parse(&args(&[])), Ok(None));
        assert_eq!(
            parse(&args(&["import", "--seen", "a.db"])),
            Ok(Some(ImportArgs {
                seen: "a.db".into(),
                candidates: None
            }))
        );
        assert!(parse(&args(&["import"])).is_err())
    }
    #[test]
    fn report() {
        let r = ImportReport {
            inbox: 1,
            discarded: 2,
            duplicates: 3,
            dismissed: 4,
            invalid: 5,
        };
        assert_eq!(
            format_report(&r),
            "inbox: 1\ndiscarded: 2\nduplicates: 3\ndismissed: 4\ninvalid: 5"
        )
    }
}
