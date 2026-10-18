use super::{App, FormAction, BOX_GAP, BOX_PADDING};
use crate::patient::Session;
use crate::ui::*;

struct SessionRow {
    idx: usize,
    date: String,
    date_w: i32,
    notes: Vec<String>,
    height: i32,
}

const EDIT_BUTTON_W: i32 = 96;
const DELETE_BUTTON_W: i32 = 120;
const ROW_BUTTON_H: i32 = 30;

pub(super) fn is_valid_session_date(date: &str) -> bool {
    chrono::NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d").is_ok()
}

impl App {
    pub(super) fn save_session(&mut self) {
        let notes = self.new_session_notes.trim().to_string();
        let date = self.new_session_date.trim().to_string();
        if !notes.is_empty() && is_valid_session_date(&date) {
            match self.editing_session.and_then(|i| self.active_patient.sessions.get_mut(i)) {
                Some(session) => {
                    session.date = date;
                    session.notes = notes;
                }
                None => {
                    self.active_patient.sessions.push(Session {
                        id: uuid::Uuid::new_v4().to_string(),
                        date,
                        notes,
                    });
                }
            }
            self.active_patient.sort_sessions();
            self.new_session_notes.clear();
            self.editing_session = None;
            self.show_session_modal = false;
        }
    }

    pub(super) fn delete_session(&mut self, idx: usize) {
        if idx < self.active_patient.sessions.len() {
            self.active_patient.sessions.remove(idx);
        }
        self.confirm_delete_session = None;
    }

    /// Draws the "Sessions recap" box at (x, y) and returns its height and any requested action.
    pub(super) fn draw_sessions(&mut self, gui: &mut Gui, fonts: &Fonts, x: i32, y: i32, w: i32) -> (i32, FormAction) {
        let bx = x + BOX_PADDING;
        let inner_w = w - 2 * BOX_PADDING;
        let mut action = FormAction::None;

        let section_header_h = 36;
        let rows: Vec<SessionRow> = self
            .active_patient
            .sessions
            .iter()
            .enumerate()
            .map(|(idx, s)| {
                let (date_w, _) = measure(&fonts.label, &s.date);
                let notes_w = inner_w - 2 * 14 - date_w - 12 - DELETE_BUTTON_W - 8 - EDIT_BUTTON_W - 12;
                let notes = wrap(&fonts.body, &format!("\"{}\"", s.notes), notes_w);
                let text_h = notes.len() as i32 * line_height(&fonts.body);
                let height = text_h.max(ROW_BUTTON_H) + 2 * 10;
                SessionRow { idx, date: s.date.clone(), date_w, notes, height }
            })
            .collect();
        let row_gap = 8;
        let content_h = rows.iter().map(|r| r.height).sum::<i32>() + row_gap * (rows.len() as i32 - 1).max(0);
        let list_h = if rows.is_empty() { 24 } else { content_h.min(220) };
        let sessions_h = 2 * BOX_PADDING + section_header_h + BOX_GAP + list_h;
        gui.fill_bordered(Rect::new(x, y, w, sessions_h), BOX_BG, BORDER);
        let header_y = y + BOX_PADDING;
        let (_, title_h) = measure(&fonts.section, "Sessions recap");
        gui.text(&fonts.section, "Sessions recap", bx, header_y + (section_header_h - title_h) / 2, TITLE);
        if gui.button(
            Rect::new(bx + inner_w - 190, header_y, 190, section_header_h),
            "+ Nouvelle séance",
            &fonts.button,
            ButtonStyle::Outline,
        ) {
            action = FormAction::OpenSessionModal;
        }

        let list = Rect::new(bx, header_y + section_header_h + BOX_GAP, inner_w, list_h);
        if rows.is_empty() {
            gui.text(&fonts.body, "Aucune séance enregistrée.", list.x, list.y + 4, PLACEHOLDER);
        } else {
            let delta = gui.take_scroll(list);
            self.sessions_scroll = clamp_scroll(self.sessions_scroll, delta, content_h, list.h);
            gui.push_clip(list);
            let mut iy = list.y - self.sessions_scroll;
            for row in &rows {
                let item = Rect::new(list.x, iy, list.w, row.height);
                gui.fill_bordered(item, WHITE, BORDER);
                gui.text(&fonts.label, &row.date, item.x + 14, item.y + 10, SESSION_DATE);
                let notes_x = item.x + 14 + row.date_w + 12;
                for (i, line) in row.notes.iter().enumerate() {
                    gui.text(&fonts.body, line, notes_x, item.y + 10 + i as i32 * line_height(&fonts.body), LABEL);
                }
                let confirming = self.confirm_delete_session == Some(row.idx);
                let (delete_label, delete_style) = if confirming {
                    ("Confirmer ?", ButtonStyle::DangerConfirm)
                } else {
                    ("Supprimer", ButtonStyle::Danger)
                };
                if gui.button(
                    Rect::new(item.right() - 14 - EDIT_BUTTON_W - 8 - DELETE_BUTTON_W, item.y + 10, DELETE_BUTTON_W, ROW_BUTTON_H),
                    delete_label,
                    &fonts.button,
                    delete_style,
                ) {
                    if confirming {
                        action = FormAction::DeleteSession(row.idx);
                    } else {
                        self.confirm_delete_session = Some(row.idx);
                    }
                }
                if gui.button(
                    Rect::new(item.right() - 14 - EDIT_BUTTON_W, item.y + 10, EDIT_BUTTON_W, ROW_BUTTON_H),
                    "Modifier",
                    &fonts.button,
                    ButtonStyle::Outline,
                ) {
                    action = FormAction::EditSession(row.idx);
                }
                iy += row.height + row_gap;
            }
            gui.scrollbar(Rect::new(list.right() - 6, list.y, 4, list.h), self.sessions_scroll, content_h);
            gui.pop_clip();
            gui.outline(list, INPUT_BORDER);
        }

        (sessions_h, action)
    }
}
