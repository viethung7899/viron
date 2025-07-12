use crate::input::actions::ActionDefinition;

#[derive(Debug, Clone)]
pub enum Operator {
    Move,
    Delete,
    Change,
    Yank,
}

#[derive(Debug, Clone)]
pub struct ComboInput {
    pub operator: Operator,
    pub operator_count: usize,
    pub motion: ActionDefinition,
    pub motion_count: usize,
}
