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
    /// Birth date as `dd/mm/yyyy`.
    pub birth_date: String,
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

/// Whether `text` is a real date written as `dd/mm/yyyy`.
pub fn is_valid_date(text: &str) -> bool {
    text.len() == 10 && chrono::NaiveDate::parse_from_str(text, "%d/%m/%Y").is_ok()
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
    fn test_date_validation() {
        assert!(is_valid_date("01/01/1990"));
        assert!(is_valid_date("29/02/2024"));
        assert!(!is_valid_date("29/02/2023"));
        assert!(!is_valid_date("01/0/1990"));
        assert!(!is_valid_date("1990-01-01"));
        assert!(!is_valid_date(""));
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
