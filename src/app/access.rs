use sdl3::keyboard::Keycode;

use super::{Access, App};
use crate::focus::Field;
use crate::patient::Patient;
use crate::storage::{create_vault, save_patients, unlock, Vault};
use crate::ui::*;

const MIN_PASSWORD_LEN: usize = 8;
const FIELD_H: i32 = 40;
const SETUP_INTRO: &str = "Vos données de patients seront chiffrées avec un mot de passe. Sans ce mot de passe, elles ne pourront pas être récupérées. Conservez-le en lieu sûr.";
const UNLOCK_INTRO: &str = "Saisissez le mot de passe pour ouvrir les données des patients.";

impl App {
    pub(super) fn draw_access(&mut self, gui: &mut Gui, fonts: &Fonts, screen: Rect) {
        let enter = gui.input.keys.contains(&Keycode::Return) || gui.input.keys.contains(&Keycode::KpEnter);
        let setup = self.access == Access::Setup;
        let (title, intro, action_label) = if setup {
            ("Protéger vos données", SETUP_INTRO, "Créer")
        } else {
            ("Déverrouiller les données", UNLOCK_INTRO, "Déverrouiller")
        };
        gui.fill(screen, BACKDROP);

        let pad = 24;
        let dialog_w = 480.min(screen.w * 9 / 10);
        let inner_w = dialog_w - 2 * pad;
        let title_h = line_height(&fonts.modal_title);
        let body_lh = line_height(&fonts.body);
        let intro_lines = wrap(&fonts.body, intro, inner_w);
        let error_lines = self
            .access_error
            .as_ref()
            .map(|error| wrap(&fonts.body, error, inner_w))
            .unwrap_or_default();
        let field_count = if setup { 2 } else { 1 };
        let field_block_h = body_lh + 6 + FIELD_H + 16;
        let error_h = if error_lines.is_empty() { 0 } else { error_lines.len() as i32 * body_lh + 16 };
        let button_h = 40;
        let dialog_h = pad
            + title_h
            + 16
            + intro_lines.len() as i32 * body_lh
            + 20
            + field_count * field_block_h
            + error_h
            + 8
            + button_h
            + pad;
        let dialog = Rect::new(
            screen.x + (screen.w - dialog_w) / 2,
            screen.y + (screen.h - dialog_h) / 2,
            dialog_w,
            dialog_h,
        );
        gui.fill_bordered(dialog, WHITE, BORDER);

        let x = dialog.x + pad;
        let mut y = dialog.y + pad;
        gui.text(&fonts.modal_title, title, x, y, TITLE);
        y += title_h + 16;
        for line in &intro_lines {
            gui.text(&fonts.body, line, x, y, TEXT);
            y += body_lh;
        }
        y += 20;

        if gui.focus.is_none() {
            gui.focus = Some(Field::Password);
        }
        gui.text(&fonts.body, "Mot de passe", x, y, LABEL);
        y += body_lh + 6;
        let placeholder = if setup { "8 caractères minimum" } else { "" };
        gui.text_field(
            Field::Password,
            Rect::new(x, y, inner_w, FIELD_H),
            &mut self.password,
            placeholder,
            &fonts.body,
            Charset::Secret,
        );
        y += FIELD_H + 16;

        if setup {
            gui.text(&fonts.body, "Confirmation", x, y, LABEL);
            y += body_lh + 6;
            gui.text_field(
                Field::PasswordConfirm,
                Rect::new(x, y, inner_w, FIELD_H),
                &mut self.password_confirm,
                "",
                &fonts.body,
                Charset::Secret,
            );
            y += FIELD_H + 16;
        }

        for line in &error_lines {
            gui.text(&fonts.body, line, x, y, ALERT_TEXT);
            y += body_lh;
        }
        if !error_lines.is_empty() {
            y += 16;
        }

        let quit_rect = Rect::new(dialog.right() - pad - 110, y + 8, 110, button_h);
        let action_rect = Rect::new(quit_rect.x - 12 - 160, y + 8, 160, button_h);
        if gui.button(quit_rect, "Quitter", &fonts.button, ButtonStyle::Secondary) {
            self.quit_requested = true;
        }
        let pressed = gui.button(action_rect, action_label, &fonts.button, ButtonStyle::Primary);
        if pressed || enter {
            self.submit_access(setup);
        }
    }

    fn submit_access(&mut self, setup: bool) {
        if !setup {
            match unlock(&self.password) {
                Ok((vault, patients)) => self.open_vault(vault, patients),
                Err(error) => self.access_error = Some(error.to_string()),
            }
            return;
        }
        if self.password.chars().count() < MIN_PASSWORD_LEN {
            self.access_error = Some(format!(
                "Le mot de passe doit contenir au moins {MIN_PASSWORD_LEN} caractères."
            ));
            return;
        }
        if self.password != self.password_confirm {
            self.access_error = Some("Les deux mots de passe ne sont pas identiques.".into());
            return;
        }
        match create_vault(&self.password) {
            Ok((vault, patients)) => self.open_vault(vault, patients),
            Err(error) => self.access_error = Some(error.to_string()),
        }
    }

    fn open_vault(&mut self, vault: Vault, patients: Vec<crate::patient::Patient>) {
        self.vault = Some(vault);
        self.patients = patients;
        self.patients.iter_mut().for_each(Patient::normalize);
        self.password.clear();
        self.password_confirm.clear();
        self.access_error = None;
        self.access = Access::Ready;
    }

    pub(super) fn save(&self, patients: &[crate::patient::Patient]) -> std::io::Result<()> {
        match &self.vault {
            Some(vault) => save_patients(vault, patients),
            None => Err(std::io::Error::other("coffre non ouvert")),
        }
    }
}
