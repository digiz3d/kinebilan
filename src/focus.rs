#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Field {
    Password,
    PasswordConfirm,
    Search,
    LastName,
    FirstName,
    BirthDate,
    CareType,
    Antecedents,
    Symptoms,
    TreatmentObjective,
    SessionDate,
    SessionNotes,
    SessionObjectives,
    SessionContent,
}

const FORM_TAB_ORDER: [Field; 7] = [
    Field::LastName,
    Field::FirstName,
    Field::BirthDate,
    Field::CareType,
    Field::Antecedents,
    Field::Symptoms,
    Field::TreatmentObjective,
];

const SESSION_TAB_ORDER: [Field; 4] = [
    Field::SessionDate,
    Field::SessionNotes,
    Field::SessionObjectives,
    Field::SessionContent,
];

const ACCESS_TAB_ORDER: [Field; 2] = [Field::Password, Field::PasswordConfirm];

impl Field {
    fn tab_order(self) -> &'static [Field] {
        if FORM_TAB_ORDER.contains(&self) {
            &FORM_TAB_ORDER
        } else if SESSION_TAB_ORDER.contains(&self) {
            &SESSION_TAB_ORDER
        } else if ACCESS_TAB_ORDER.contains(&self) {
            &ACCESS_TAB_ORDER
        } else {
            &[]
        }
    }

    pub fn tab_step(self, forward: bool) -> Field {
        let order = self.tab_order();
        let len = order.len();
        match order.iter().position(|&f| f == self) {
            Some(i) if forward => order[(i + 1) % len],
            Some(i) => order[(i + len - 1) % len],
            None => self,
        }
    }
}
