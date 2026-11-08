#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Field {
    Password,
    PasswordConfirm,
    Search,
    LastName,
    FirstName,
    BirthDate,
    CareType,
    CreationDate,
    Antecedents,
    Symptoms,
    TreatmentObjective,
    SessionDate,
    SessionNotes,
    SessionObjectives,
    SessionContent,
}

const FORM_TAB_ORDER: [Field; 8] = [
    Field::LastName,
    Field::FirstName,
    Field::BirthDate,
    Field::CareType,
    Field::CreationDate,
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

    #[allow(dead_code)]
    pub fn tab_order_index(self) -> Option<usize> {
        self.tab_order().iter().position(|&field| field == self)
    }

    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::Field;

    #[test]
    fn form_orders_are_visual_and_wrap_in_both_directions() {
        for (order, first, last) in [
            (
                &[
                    Field::LastName,
                    Field::FirstName,
                    Field::BirthDate,
                    Field::CareType,
                    Field::CreationDate,
                    Field::Antecedents,
                    Field::Symptoms,
                    Field::TreatmentObjective,
                ][..],
                Field::LastName,
                Field::TreatmentObjective,
            ),
            (
                &[
                    Field::SessionDate,
                    Field::SessionNotes,
                    Field::SessionObjectives,
                    Field::SessionContent,
                ][..],
                Field::SessionDate,
                Field::SessionContent,
            ),
            (
                &[Field::Password, Field::PasswordConfirm][..],
                Field::Password,
                Field::PasswordConfirm,
            ),
        ] {
            for (index, field) in order.iter().enumerate() {
                assert_eq!(field.tab_order_index(), Some(index));
            }
            assert_eq!(first.tab_step(false), last);
            assert_eq!(last.tab_step(true), first);
        }
    }
}
