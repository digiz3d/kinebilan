use super::App;
use crate::focus::Field;
use crate::ui::*;

impl App {
    /// Draws the identity fields at (x, y) and returns the y coordinate below them.
    pub(super) fn draw_identity(&mut self, gui: &mut Gui, fonts: &Fonts, x: i32, y: i32, w: i32) -> i32 {
        let gap = 16;
        let col_w = (w - 2 * gap) / 3;
        let cols = [x, x + col_w + gap, x + 2 * (col_w + gap)];
        let label_h = line_height(&fonts.label);
        let field_h = 40;
        gui.text(&fonts.label, "Nom *", cols[0], y, LABEL);
        gui.text(&fonts.label, "Prénom *", cols[1], y, LABEL);
        gui.text(&fonts.label, "Date de naissance *", cols[2], y, LABEL);
        let field_y = y + label_h + 6;
        if gui.text_field(
            Field::LastName,
            Rect::new(cols[0], field_y, col_w, field_h),
            &mut self.active_patient.last_name,
            "Nom de famille",
            &fonts.body,
            Charset::Text,
        ) {
            self.validation_error = None;
        }
        if gui.text_field(
            Field::FirstName,
            Rect::new(cols[1], field_y, col_w, field_h),
            &mut self.active_patient.first_name,
            "Prénom",
            &fonts.body,
            Charset::Text,
        ) {
            self.validation_error = None;
        }
        if gui.text_field(
            Field::BirthDate,
            Rect::new(cols[2], field_y, col_w, field_h),
            &mut self.active_patient.birth_date,
            "AAAA-MM-JJ",
            &fonts.body,
            Charset::Date,
        ) {
            self.validation_error = None;
        }
        field_y + field_h + 20
    }
}
