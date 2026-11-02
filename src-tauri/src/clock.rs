use std::time::{SystemTime, UNIX_EPOCH};

/// A moment with its UTC offset, as job sources publish it.
pub type Now = chrono::DateTime<chrono::FixedOffset>;

pub fn now() -> Now {
    chrono::Local::now().fixed_offset()
}

/// The fixed clock every crawler test shares: 2026-11-02 12:00 in São Paulo.
#[cfg(test)]
pub fn fixed_now() -> Now {
    chrono::DateTime::parse_from_rfc3339("2026-11-02T12:00:00-03:00").unwrap()
}
pub fn today() -> String {
    let days = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 86_400) as i64;
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let y = y + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn today_is_iso_date() {
        assert_eq!(today().len(), 10);
        assert_eq!(&today()[4..5], "-");
        assert_eq!(&today()[7..8], "-");
    }
}
