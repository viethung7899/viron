use crate::actions::{ActionContext, ActionResult};
use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait(?Send)]
pub trait Executable: Debug {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult;
}

#[derive(Debug)]
pub struct CompositeExecutable(Vec<Box<dyn Executable>>);

impl CompositeExecutable {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, action: impl Executable + 'static) -> &mut Self {
        self.0.push(Box::new(action));
        self
    }
}

#[async_trait(?Send)]
impl Executable for CompositeExecutable {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        for action in &self.0 {
            action.execute(ctx).await?;
        }
        Ok(())
    }
}
