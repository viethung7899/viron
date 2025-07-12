use crate::core::operation::Operator;
use crate::input::actions::{Action, ActionContext, ActionResult, Executable};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct RepeatedAction {
    repeat: usize,
    action: Box<dyn Action>,
}

impl RepeatedAction {
    pub fn new(repeat: usize, action: Box<dyn Action>) -> Self {
        Self { repeat, action }
    }
}

#[async_trait(?Send)]
impl Executable for RepeatedAction {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        for _ in 0..self.repeat {
            self.action.execute(ctx).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ComboAction {
    operator: Operator,
    repeat: usize,
    motion: Box<dyn Action>,
}

impl ComboAction {
    pub fn new(operator: Operator, repeat: usize, motion: Box<dyn Action>) -> Self {
        Self {
            operator,
            repeat,
            motion,
        }
    }
}

#[async_trait(?Send)]
impl Executable for ComboAction {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        todo!()
    }
}
