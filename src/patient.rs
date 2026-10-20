use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub id: String,
    pub date: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub objectives: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Patient {
    pub id: String,
    pub last_name: String,
    pub first_name: String,
    /// Birth date as `yyyy-mm-dd`.
    pub birth_date: String,
    /// Date and time the patient was created, in UTC as `yyyy-mm-ddTHH:MM:00Z`. Empty when unknown.
    pub creation_date: String,
    #[serde(default)]
    pub care_type: String,
    #[serde(default, alias = "prior_issues")]
    pub antecedents: String,
    #[serde(default)]
    pub symptoms: String,
    #[serde(default)]
    pub treatment_objective: String,
    pub sessions: Vec<Session>,
}

impl Patient {
    /// Sorts sessions by date, most recent first. Dates are `YYYY-MM-DD`, so string order is date order.
    pub fn sort_sessions(&mut self) {
        self.sessions.sort_by(|a, b| b.date.cmp(&a.date));
    }

    /// Normalizes fields that require a fixed format: the last name is always uppercase,
    /// the first name always starts with an uppercase letter.
    pub fn normalize(&mut self) {
        self.last_name = self.last_name.trim().to_uppercase();
        self.first_name = capitalize_first(&self.first_name);
    }

    /// The first name as it should be displayed, with an uppercase first letter.
    pub fn display_first_name(&self) -> String {
        capitalize_first(&self.first_name)
    }
}

/// Parses a date written as `dd/mm/yyyy`, the format shown and typed in the app.
fn parse_display_date(text: &str) -> Option<chrono::NaiveDate> {
    if text.len() != 10 {
        return None;
    }
    chrono::NaiveDate::parse_from_str(text, "%d/%m/%Y").ok()
}

/// Converts a `dd/mm/yyyy` date to the stored `yyyy-mm-dd` form. `None` when it is not a real date.
pub fn to_storage_date(display: &str) -> Option<String> {
    parse_display_date(display).map(|date| date.format("%Y-%m-%d").to_string())
}

/// Converts a stored `yyyy-mm-dd` date to `dd/mm/yyyy` for display. Text that is not a date is returned unchanged.
pub fn to_display_date(stored: &str) -> String {
    match chrono::NaiveDate::parse_from_str(stored, "%Y-%m-%d") {
        Ok(date) => date.format("%d/%m/%Y").to_string(),
        Err(_) => stored.to_string(),
    }
}

/// Formats a UTC instant as it is stored: `yyyy-mm-ddTHH:MM:00Z`. Seconds are dropped.
pub fn format_stored_datetime(instant: DateTime<Utc>) -> String {
    instant.format("%Y-%m-%dT%H:%M:00Z").to_string()
}

/// Converts a local `dd/mm/yyyy HH:MM` to the stored UTC form. `None` when it is not a real date and time.
pub fn to_storage_datetime(display: &str) -> Option<String> {
    if display.len() != 16 {
        return None;
    }
    let naive = NaiveDateTime::parse_from_str(display, "%d/%m/%Y %H:%M").ok()?;
    let local = Local.from_local_datetime(&naive).earliest()?;
    Some(format_stored_datetime(local.with_timezone(&Utc)))
}

/// Converts a stored UTC date and time to local `dd/mm/yyyy HH:MM` for display. Text that is not a date and time is returned unchanged.
pub fn to_display_datetime(stored: &str) -> String {
    match DateTime::parse_from_rfc3339(stored) {
        Ok(instant) => instant.with_timezone(&Local).format("%d/%m/%Y %H:%M").to_string(),
        Err(_) => stored.to_string(),
    }
}

/// Uppercases the first letter of `s`, leaving the rest untouched.
fn capitalize_first(s: &str) -> String {
    let s = s.trim_start();
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patient_validation() {
        let invalid1 = Patient {
            id: "1".into(),
            last_name: "".into(),
            first_name: "Jean".into(),
            birth_date: "01/01/1990".into(),
            ..Default::default()
        };
        let is_valid = |p: &Patient| {
            !p.last_name.trim().is_empty()
                && !p.first_name.trim().is_empty()
                && !p.birth_date.trim().is_empty()
        };
        assert!(!is_valid(&invalid1));

        let valid = Patient {
            id: "2".into(),
            last_name: "Dupont".into(),
            first_name: "Jean".into(),
            birth_date: "01/01/1990".into(),
            ..Default::default()
        };
        assert!(is_valid(&valid));
    }

    #[test]
    fn test_date_conversion() {
        assert_eq!(to_storage_date("12/04/1985").as_deref(), Some("1985-04-12"));
        assert_eq!(to_storage_date("29/02/2023"), None);
        assert_eq!(to_storage_date("1985-04-12"), None);
        assert_eq!(to_display_date("1985-04-12"), "12/04/1985");
        assert_eq!(to_display_date(""), "");
        assert_eq!(to_display_date("12/04/1985"), "12/04/1985");
    }

    #[test]
    fn test_datetime_round_trip() {
        let stored = to_storage_datetime("20/10/2026 16:15").unwrap();
        assert!(stored.ends_with("Z") && stored.len() == 20);
        assert_eq!(to_display_datetime(&stored), "20/10/2026 16:15");
        assert_eq!(to_storage_datetime("20/10/2026"), None);
        assert_eq!(to_storage_datetime("31/02/2026 10:00"), None);
        assert_eq!(to_storage_datetime("20/10/2026 25:00"), None);
        assert_eq!(to_display_datetime(""), "");
    }

    #[test]
    fn test_delete_patient_list() {
        let mut list = vec![
            Patient { id: "p-1".into(), ..Default::default() },
            Patient { id: "p-2".into(), ..Default::default() },
        ];
        list.retain(|p| p.id != "p-1");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "p-2");
    }
}
