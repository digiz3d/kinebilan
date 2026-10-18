use super::{App, BOX_GAP, BOX_PADDING};
use crate::focus::Field;
use crate::ui::*;

impl App {
    /// Draws the "Bilan clinique" box at (x, y) and returns its height.
    pub(super) fn draw_clinical(&mut self, gui: &mut Gui, fonts: &Fonts, x: i32, y: i32, w: i32) -> i32 {
        let label_h = line_height(&fonts.label);
        let inner_w = w - 2 * BOX_PADDING;
        let heading_h = line_height(&fonts.section);
        let group_pitch = label_h + 6;
        let areas_h = 3 * group_pitch
            + area_height(&fonts.body, &self.active_patient.antecedents, inner_w, 3)
            + area_height(&fonts.body, &self.active_patient.symptoms, inner_w, 3)
            + area_height(&fonts.body, &self.active_patient.treatment_objective, inner_w, 3)
            + 2 * BOX_GAP;
        let clinical_h = 2 * BOX_PADDING + heading_h + BOX_GAP + areas_h;
        gui.fill_bordered(Rect::new(x, y, w, clinical_h), BOX_BG, BORDER);
        let bx = x + BOX_PADDING;
        let mut by = y + BOX_PADDING;
        gui.text(&fonts.section, "Bilan clinique", bx, by, TITLE);
        by += heading_h + BOX_GAP;

        gui.text(&fonts.label, "Antécédents", bx, by, LABEL);
        by += group_pitch;
        by += gui.text_area(
            Field::Antecedents,
            bx,
            by,
            inner_w,
            &mut self.active_patient.antecedents,
            "Antécédents médicaux, chirurgicaux, traumatismes...",
            &fonts.body,
            3,
        ) + BOX_GAP;

        gui.text(&fonts.label, "Symptômes", bx, by, LABEL);
        by += group_pitch;
        by += gui.text_area(
            Field::Symptoms,
            bx,
            by,
            inner_w,
            &mut self.active_patient.symptoms,
            "Douleurs, limitations articulaires, gênes fonctionnelles...",
            &fonts.body,
            3,
        ) + BOX_GAP;

        gui.text(&fonts.label, "Objectif de la prise en charge", bx, by, LABEL);
        by += group_pitch;
        gui.text_area(
            Field::TreatmentObjective,
            bx,
            by,
            inner_w,
            &mut self.active_patient.treatment_objective,
            "Objectif global du traitement de rééducation...",
            &fonts.body,
            3,
        );

        clinical_h
    }
}
