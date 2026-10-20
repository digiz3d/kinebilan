use chrono::{DateTime, NaiveDate, Utc};

use super::App;
use crate::patient::Patient;

impl App {
    pub(super) fn visible_patients(&self) -> Vec<usize> {
        let query = fold_accents(&self.search_query);
        let mut visible: Vec<usize> = self
            .patients
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                query.trim().is_empty()
                    || fold_accents(&p.last_name).contains(&query)
                    || fold_accents(&p.first_name).contains(&query)
                    || fold_accents(&p.care_type).contains(&query)
            })
            .map(|(i, _)| i)
            .collect();
        visible.sort_by(|&a, &b| last_activity_date(&self.patients[b]).cmp(&last_activity_date(&self.patients[a])));
        visible
    }
}

/// The latest session day (at midnight) or the creation instant, whichever is later. Creation keeps its time,
/// so two patients created on the same day are still ordered.
fn last_activity_date(patient: &Patient) -> Option<DateTime<Utc>> {
    let latest_session = patient
        .sessions
        .iter()
        .filter_map(|s| NaiveDate::parse_from_str(&s.date, "%Y-%m-%d").ok())
        .max()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|midnight| midnight.and_utc());
    let created = DateTime::parse_from_rfc3339(&patient.creation_date).ok().map(|created| created.with_timezone(&Utc));
    latest_session.into_iter().chain(created).max()
}

fn fold_accents(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars().flat_map(char::to_lowercase) {
        let base = match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => "a",
            'ç' => "c",
            'è' | 'é' | 'ê' | 'ë' => "e",
            'ì' | 'í' | 'î' | 'ï' => "i",
            'ñ' => "n",
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => "o",
            'ù' | 'ú' | 'û' | 'ü' => "u",
            'ý' | 'ÿ' => "y",
            'œ' => "oe",
            'æ' => "ae",
            _ => {
                out.push(c);
                continue;
            }
        };
        out.push_str(base);
    }
    out
}
