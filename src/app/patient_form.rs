use super::form_bar::BAR_H;
use super::form_footer::FOOTER_H;
use super::{App, FormAction, Page, CARD_PADDING};
use crate::focus::Field;
use crate::patient::to_display_date;
use crate::ui::*;

impl App {
    pub(super) fn draw_form(&mut self, gui: &mut Gui, fonts: &Fonts, card: Rect, screen: Rect) {
        let modal_open = self.show_session_modal || self.show_leave_modal;
        gui.set_interactive(!modal_open);
        let mut action = FormAction::None;

        let bar = Rect::new(card.x, card.y, card.w, BAR_H);
        let bar_action = self.draw_form_bar(gui, fonts, bar);
        if bar_action != FormAction::None {
            action = bar_action;
        }

        let body = Rect::new(card.x, bar.bottom(), card.w, card.h - BAR_H);
        let viewport = body.inset(CARD_PADDING);
        let (x, w) = (viewport.x, viewport.w);

        gui.push_clip(body);
        let mut y = viewport.y - self.form_scroll;

        if let Some(error) = self.validation_error.clone() {
            let alert = Rect::new(x, y, w, 44);
            gui.fill_bordered(alert, ALERT_BG, ALERT_BORDER);
            gui.text(&fonts.body, &error, x + 16, y + 12, ALERT_TEXT);
            y += 44 + 20;
        }

        y = self.draw_identity(gui, fonts, x, y, w);

        y += self.draw_clinical(gui, fonts, x, y, w) + 20;

        let sessions_fill_h = viewport.bottom() - (y + self.form_scroll) - 20 - FOOTER_H;
        let (sessions_h, sessions_action) = self.draw_sessions(gui, fonts, x, y, w, sessions_fill_h);
        if sessions_action != FormAction::None {
            action = sessions_action;
        }
        y += sessions_h + 20;

        let (footer_action, footer_bottom) = self.draw_form_footer(gui, fonts, x, y, w);
        if footer_action != FormAction::None {
            action = footer_action;
        }
        y = footer_bottom;

        let content_bottom = y + self.form_scroll;
        let content_h = content_bottom - viewport.y;
        let delta = gui.take_scroll(body);
        self.form_scroll = clamp_scroll(self.form_scroll, delta, content_h, viewport.h);
        gui.scrollbar(Rect::new(card.right() - 6, body.y, 4, body.h), viewport.h, self.form_scroll, content_h);
        gui.pop_clip();

        gui.set_interactive(true);
        if self.show_leave_modal {
            let modal_action = self.draw_leave_modal(gui, fonts, screen);
            if modal_action != FormAction::None {
                action = modal_action;
            }
        } else if self.show_session_modal {
            let modal_action = self.draw_session_modal(gui, fonts, screen);
            if modal_action != FormAction::None {
                action = modal_action;
            }
        }

        match action {
            FormAction::None => {}
            FormAction::Back => {
                if self.has_unsaved_changes() {
                    self.show_leave_modal = true;
                } else {
                    self.page = Page::Home;
                }
                gui.focus = None;
            }
            FormAction::CancelLeave => {
                self.show_leave_modal = false;
            }
            FormAction::LeaveWithoutSaving => {
                self.show_leave_modal = false;
                self.page = Page::Home;
                gui.focus = None;
            }
            FormAction::Delete => {
                self.delete_active();
                gui.focus = None;
            }
            FormAction::Save => {
                if self.save_active() {
                    gui.focus = None;
                }
            }
            FormAction::SaveAndLeave => {
                self.show_leave_modal = false;
                if self.save_active() {
                    self.page = Page::Home;
                    gui.focus = None;
                }
            }
            FormAction::OpenSessionModal => {
                self.new_session_date = chrono::Local::now().format("%d/%m/%Y").to_string();
                self.new_session_notes.clear();
                self.new_session_objectives.clear();
                self.new_session_content.clear();
                self.editing_session = None;
                self.show_session_modal = true;
                gui.focus = Some(Field::SessionNotes);
            }
            FormAction::EditSession(idx) => {
                let session = &self.active_patient.sessions[idx];
                self.new_session_date = to_display_date(&session.date);
                self.new_session_notes = session.notes.clone();
                self.new_session_objectives = session.objectives.clone();
                self.new_session_content = session.content.clone();
                self.editing_session = Some(idx);
                self.show_session_modal = true;
                gui.focus = Some(Field::SessionNotes);
            }
            FormAction::DeleteSession(idx) => {
                self.delete_session(idx);
            }
            FormAction::CloseSessionModal => {
                self.show_session_modal = false;
                self.editing_session = None;
                gui.focus = None;
            }
            FormAction::SaveSession => {
                self.save_session();
                if !self.show_session_modal {
                    gui.focus = None;
                }
            }
        }
    }
}
