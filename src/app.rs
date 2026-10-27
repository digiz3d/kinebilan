mod access;
mod clinical;
mod form_bar;
mod form_footer;
mod identity;
mod leave_modal;
mod load_error;
mod patient_actions;
mod update_notice;
mod patient_form;
mod patient_list;
mod search;
mod search_bar;
mod session_modal;
mod sessions;

use crate::patient::Patient;
use crate::storage::{stored_kind, Stored, Vault};
use crate::ui::*;

const CARD_PADDING: i32 = 28;
const BOX_PADDING: i32 = 18;
const BOX_GAP: i32 = 14;

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Home,
    PatientForm,
}

#[derive(Clone, Copy, PartialEq)]
enum Access {
    Setup,
    Unlock,
    Ready,
}

#[derive(Clone, Copy, PartialEq)]
enum FormAction {
    None,
    Back,
    Delete,
    Save,
    SaveAndLeave,
    OpenSessionModal,
    EditSession(usize),
    DeleteSession(usize),
    CloseSessionModal,
    SaveSession,
    CancelLeave,
    LeaveWithoutSaving,
}

pub struct App {
    patients: Vec<Patient>,
    load_error: Option<String>,
    installed_update: Option<String>,
    access: Access,
    vault: Option<Vault>,
    password: String,
    password_confirm: String,
    access_error: Option<String>,
    quit_requested: bool,
    page: Page,
    search_query: String,
    active_patient: Patient,
    birth_date_input: String,
    creation_date_input: String,
    validation_error: Option<String>,
    show_session_modal: bool,
    show_leave_modal: bool,
    new_session_date: String,
    new_session_notes: String,
    new_session_objectives: String,
    new_session_content: String,
    editing_session: Option<usize>,
    confirm_delete_session: Option<usize>,
    confirm_delete: bool,
    patients_scroll: i32,
    form_scroll: i32,
    sessions_scroll: i32,
}

impl App {
    pub fn new() -> Self {
        let (access, load_error) = match stored_kind() {
            Ok(Stored::Encrypted) => (Access::Unlock, None),
            Ok(Stored::Empty | Stored::Plaintext) => (Access::Setup, None),
            Err(error) => (Access::Setup, Some(error.to_string())),
        };
        Self {
            patients: Vec::new(),
            load_error,
            installed_update: None,
            access,
            vault: None,
            password: String::new(),
            password_confirm: String::new(),
            access_error: None,
            quit_requested: false,
            page: Page::Home,
            search_query: String::new(),
            active_patient: Patient::default(),
            birth_date_input: String::new(),
            creation_date_input: String::new(),
            validation_error: None,
            show_session_modal: false,
            show_leave_modal: false,
            new_session_date: String::new(),
            new_session_notes: String::new(),
            new_session_objectives: String::new(),
            new_session_content: String::new(),
            editing_session: None,
            confirm_delete_session: None,
            confirm_delete: false,
            patients_scroll: 0,
            form_scroll: 0,
            sessions_scroll: 0,
        }
    }

    pub fn quit_requested(&self) -> bool {
        self.quit_requested
    }

    pub fn set_installed_update(&mut self, version: String) {
        self.installed_update = Some(version);
    }

    pub fn draw(&mut self, gui: &mut Gui, fonts: &Fonts, screen: Rect) {
        gui.fill(screen, BG);
        let card = Rect::new(0, 0, screen.w, screen.h);
        gui.fill_bordered(card, WHITE, BORDER);

        if self.load_error.is_some() {
            self.draw_load_error(gui, fonts, screen);
            return;
        }
        if self.installed_update.is_some() {
            self.draw_update_notice(gui, fonts, screen);
            return;
        }
        if self.access != Access::Ready {
            self.draw_access(gui, fonts, screen);
            return;
        }

        match self.page {
            Page::Home => self.draw_home(gui, fonts, card.inset(CARD_PADDING)),
            Page::PatientForm => self.draw_form(gui, fonts, card, screen),
        }
    }
}
