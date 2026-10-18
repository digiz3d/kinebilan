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
}
