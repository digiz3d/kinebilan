use super::App;
use crate::ui::*;

impl App {
    pub(super) fn draw_home(&mut self, gui: &mut Gui, fonts: &Fonts, area: Rect) {
        let header_h = 56;
        gui.text(&fonts.title, "KinéBilan", area.x, area.y, TITLE);
        gui.text(&fonts.body, "Dossiers des patients", area.x, area.y + 36, MUTED);
        let new_button = Rect::new(area.right() - 190, area.y + 8, 190, 40);
        if gui.button(new_button, "Nouveau patient", &fonts.button, ButtonStyle::Primary) {
            self.start_new_patient();
            gui.focus = None;
            return;
        }

        let divider_y = area.y + header_h + 16;
        gui.fill(Rect::new(area.x, divider_y, area.w, 1), DIVIDER);
        let mut y = divider_y + 21;

        self.draw_search(gui, fonts, Rect::new(area.x, y, area.w, 44));
        y += 44 + 20;

        let list = Rect::new(area.x, y, area.w, area.bottom() - y);
        let visible = self.visible_patients();

        if visible.is_empty() {
            gui.text_centered(
                &fonts.body,
                "Aucun patient trouvé.",
                Rect::new(list.x, list.y, list.w, 96),
                PLACEHOLDER,
            );
            return;
        }

        let item_h = 70;
        let pitch = item_h + 10;
        let content_h = visible.len() as i32 * pitch - 10;
        let delta = gui.take_scroll(list);
        self.patients_scroll = clamp_scroll(self.patients_scroll, delta, content_h, list.h);

        let mut open: Option<usize> = None;
        gui.push_clip(list);
        for (row, &index) in visible.iter().enumerate() {
            let item = Rect::new(list.x, list.y + row as i32 * pitch - self.patients_scroll, list.w, item_h);
            let hot = gui.hovered(item);
            gui.fill_bordered(
                item,
                if hot { ACCENT_SOFT } else { ITEM_BG },
                if hot { ACCENT_BORDER_SOFT } else { BORDER },
            );

            let p = &self.patients[index];
            let name = format!("{} {}", p.last_name.to_uppercase(), p.display_first_name());
            gui.text(&fonts.section, &name, item.x + 18, item.y + 14, TEXT);
            if !p.care_type.trim().is_empty() {
                let (name_w, name_h) = measure(&fonts.section, &name);
                let (care_w, _) = measure(&fonts.badge, &p.care_type);
                let tag_h = 20;
                let tag = Rect::new(
                    item.x + 18 + name_w + 10,
                    item.y + 14 + (name_h - tag_h) / 2,
                    care_w + 16,
                    tag_h,
                );
                gui.fill(tag, SECONDARY_BG);
                gui.text_centered(&fonts.badge, &p.care_type, tag, LABEL);
            }
            let dob = format!("Né(e) le: {}", p.birth_date);
            gui.text(&fonts.small, &dob, item.x + 18, item.y + 42, MUTED);

            let badge_label = format!("{} séance(s)", p.sessions.len());
            let (badge_text_w, _) = measure(&fonts.badge, &badge_label);
            let badge = Rect::new(item.right() - 18 - (badge_text_w + 20), item.y + (item_h - 24) / 2, badge_text_w + 20, 24);
            gui.fill(badge, BADGE_BG);
            gui.text_centered(&fonts.badge, &badge_label, badge, BADGE_TEXT);

            if gui.take_click(item) {
                open = Some(index);
            }
        }
        gui.scrollbar(Rect::new(list.right() - 6, list.y, 4, list.h), list.h, self.patients_scroll, content_h);
        gui.pop_clip();
        gui.outline(list, INPUT_BORDER);

        if let Some(index) = open {
            let patient = self.patients[index].clone();
            self.open_patient(patient);
            gui.focus = None;
        }
    }
}
