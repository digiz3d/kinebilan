use super::App;
use crate::ui::*;

impl App {
    pub(super) fn draw_update_notice(&mut self, gui: &mut Gui, fonts: &Fonts, screen: Rect) {
        let Some(version) = self.installed_update.clone() else {
            return;
        };
        gui.fill(screen, BACKDROP);

        let pad = 24;
        let dialog_w = 480.min(screen.w * 9 / 10);
        let inner_w = dialog_w - 2 * pad;
        let title_h = line_height(&fonts.modal_title);
        let body_lh = line_height(&fonts.body);
        let message = format!(
            "KinéBilan {version} a été installé. Redémarrez l'application pour utiliser la nouvelle version."
        );
        let lines = wrap(&fonts.body, &message, inner_w);
        let body_h = lines.len() as i32 * body_lh;
        let button_h = 40;
        let dialog_h = pad + title_h + 16 + body_h + 24 + button_h + pad;
        let dialog = Rect::new(
            screen.x + (screen.w - dialog_w) / 2,
            screen.y + (screen.h - dialog_h) / 2,
            dialog_w,
            dialog_h,
        );
        gui.fill_bordered(dialog, WHITE, BORDER);

        let x = dialog.x + pad;
        let mut y = dialog.y + pad;
        gui.text(&fonts.modal_title, "Mise à jour installée", x, y, TITLE);
        y += title_h + 16;
        for line in &lines {
            gui.text(&fonts.body, line, x, y, TEXT);
            y += body_lh;
        }

        let ok_rect = Rect::new(dialog.right() - pad - 110, y + 24, 110, button_h);
        if gui.button(ok_rect, "OK", &fonts.button, ButtonStyle::Primary) {
            self.installed_update = None;
        }
    }
}
