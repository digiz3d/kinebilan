mod clinical;
mod form_footer;
mod identity;
mod patient_actions;
mod patient_form;
mod patient_list;
mod search;
mod search_bar;
mod session_modal;
mod sessions;

use crate::patient::Patient;
use crate::storage::load_patients;
use crate::ui::*;

const CARD_PADDING: i32 = 28;
const BOX_PADDING: i32 = 18;
const BOX_GAP: i32 = 14;
const MAX_CONTENT_WIDTH: i32 = 900;

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Home,
    PatientForm,
}

#[derive(Clone, Copy, PartialEq)]
enum FormAction {
    None,
    Back,
    Delete,
    Save,
    OpenSessionModal,
    CloseSessionModal,
    AddSession,
}

pub struct App {
    patients: Vec<Patient>,
    page: Page,
    search_query: String,
    active_patient: Patient,
    validation_error: Option<String>,
    show_session_modal: bool,
    new_session_notes: String,
    confirm_delete: bool,
    patients_scroll: i32,
    form_scroll: i32,
    sessions_scroll: i32,
}

impl App {
    pub fn new() -> Self {
        Self {
            patients: load_patients(),
            page: Page::Home,
            search_query: String::new(),
            active_patient: Patient::default(),
            validation_error: None,
            show_session_modal: false,
            new_session_notes: String::new(),
            confirm_delete: false,
            patients_scroll: 0,
            form_scroll: 0,
            sessions_scroll: 0,
        }
    }

    pub fn draw(&mut self, gui: &mut Gui, fonts: &Fonts, screen: Rect) {
        gui.fill(screen, BG);
        let card_w = (screen.w - 32).min(MAX_CONTENT_WIDTH);
        let card = Rect::new((screen.w - card_w) / 2, 24, card_w, (screen.h - 48).max(0));
        gui.fill_bordered(card, WHITE, BORDER);

        match self.page {
            Page::Home => self.draw_home(gui, fonts, card.inset(CARD_PADDING)),
            Page::PatientForm => self.draw_form(gui, fonts, card, screen),
        }
    }
}
