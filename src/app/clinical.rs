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
        let p = &self.active_patient;
        let areas_h: i32 = [&p.prior_issues, &p.symptoms, &p.goals]
            .iter()
            .map(|t| group_pitch + area_height(&fonts.body, t, inner_w, 3))
            .sum();
        let clinical_h = 2 * BOX_PADDING + heading_h + BOX_GAP + areas_h + 2 * BOX_GAP;
        gui.fill_bordered(Rect::new(x, y, w, clinical_h), BOX_BG, BORDER);
        let bx = x + BOX_PADDING;
        let mut by = y + BOX_PADDING;
        gui.text(&fonts.section, "Bilan clinique", bx, by, TITLE);
        by += heading_h + BOX_GAP;

        gui.text(&fonts.label, "Antécédents / Prior issues", bx, by, LABEL);
        by += group_pitch;
        by += gui.text_area(
            Field::PriorIssues,
            bx,
            by,
            inner_w,
            &mut self.active_patient.prior_issues,
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

        gui.text(&fonts.label, "Objectifs thérapeutiques", bx, by, LABEL);
        by += group_pitch;
        gui.text_area(
            Field::Goals,
            bx,
            by,
            inner_w,
            &mut self.active_patient.goals,
            "Objectifs du patient et plan de rééducation...",
            &fonts.body,
            3,
        );

        clinical_h
    }
}
