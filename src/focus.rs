#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Field {
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

impl Field {
    pub fn tab_step(self, forward: bool) -> Field {
        let len = FORM_TAB_ORDER.len();
        match FORM_TAB_ORDER.iter().position(|&f| f == self) {
            Some(i) if forward => FORM_TAB_ORDER[(i + 1) % len],
            Some(i) => FORM_TAB_ORDER[(i + len - 1) % len],
            None => self,
        }
    }
}
