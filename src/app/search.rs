use super::App;
use crate::focus::Field;
use crate::ui::*;

impl App {
    pub(super) fn draw_search(&mut self, gui: &mut Gui, fonts: &Fonts, rect: Rect) {
        gui.text_field(
            Field::Search,
            rect,
            &mut self.search_query,
            "Rechercher par nom ou prénom...",
            &fonts.body_large,
            Charset::Text,
        );
    }

    pub(super) fn visible_patients(&self) -> Vec<usize> {
        let query = fold_accents(&self.search_query);
        self.patients
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                query.trim().is_empty()
                    || fold_accents(&p.last_name).contains(&query)
                    || fold_accents(&p.first_name).contains(&query)
            })
            .map(|(i, _)| i)
            .collect()
    }
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
