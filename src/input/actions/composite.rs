use crate::core::operation::Operator;
use crate::input::actions::{
    create_action_from_definition, Action, ActionContext, ActionDefinition, ActionResult,
    Executable,
};
use async_trait::async_trait;

// A composite action that runs multiple actions in sequence
#[derive(Debug, Clone)]
pub struct CompositeAction {
    actions: Vec<Box<dyn Action>>,
    description: String,
}

impl CompositeAction {
    pub fn new(description: &str) -> Self {
        Self {
            actions: Vec::new(),
            description: description.to_string(),
        }
    }

    pub fn add(&mut self, action: Box<dyn Action>) -> &mut Self {
        self.actions.push(action);
        self
    }
}

#[async_trait(?Send)]
impl Executable for CompositeAction {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        for action in &self.actions {
            action.execute(ctx).await?;
        }
        Ok(())
    }
}

impl Action for CompositeAction {
    fn describe(&self) -> &str {
        &self.description
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::Composite {
            description: self.description.clone(),
            actions: self
                .actions
                .iter()
                .map(|action| action.to_serializable())
                .collect(),
        }
    }
    fn clone_box(&self) -> Box<(dyn Action + 'static)> {
        Box::new(self.clone())
    }
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
        for action in self.0.iter() {
            action.execute(ctx).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RepeatingAction {
    repeat: usize,
    action: ActionDefinition,
}

impl RepeatingAction {
    pub fn new(repeat: usize, action: ActionDefinition) -> Self {
        Self { repeat, action }
    }
}

#[async_trait(?Send)]
impl Executable for RepeatingAction {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let action = create_action_from_definition(&self.action);
        for _ in 0..self.repeat {
            action.execute(ctx).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ComboAction {
    operator: Operator,
    repeat: usize,
    motion: ActionDefinition,
}

impl ComboAction {
    pub fn new(operator: Operator, repeat: usize, motion: ActionDefinition) -> Self {
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
