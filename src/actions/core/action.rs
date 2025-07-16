use crate::actions::core::{ActionDefinition, Executable};
use crate::actions::{ActionContext, ActionResult};
use async_trait::async_trait;
use std::fmt::Debug;

pub trait Action: Debug + Executable {
    fn describe(&self) -> &str;
    fn to_serializable(&self) -> ActionDefinition;
    fn clone_box(&self) -> Box<dyn Action>;
}

impl Clone for Box<dyn Action> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

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

macro_rules! impl_action {
    ($action_type:ty, $description:expr, $self:ident $definition_block:block) => {
        impl crate::actions::core::Action for $action_type {
            fn clone_box(&self) -> Box<dyn crate::actions::core::Action> {
                Box::new(self.clone())
            }

            fn describe(&self) -> &str {
                $description
            }

            fn to_serializable(&$self) -> ActionDefinition $definition_block
        }
    };

    ($action_type:ty, $description:expr, $definition:expr) => {
        impl crate::actions::core::Action for $action_type {
            fn clone_box(&self) -> Box<dyn crate::actions::core::Action> {
                Box::new(self.clone())
            }

            fn describe(&self) -> &str {
                $description
            }

            fn to_serializable(&self) -> ActionDefinition {
                $definition
            }
        }
    };
}

pub(crate) use impl_action;
