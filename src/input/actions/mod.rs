use crate::config::Config;
use crate::core::buffer_manager::BufferManager;
use crate::core::command::{CommandBuffer, SearchBuffer};
use crate::core::message::MessageManager;
use crate::core::mode::Mode;
use crate::core::{cursor::Cursor, viewport::Viewport};
use crate::input::InputState;
use crate::service::LspService;
use crate::ui::components::ComponentIds;
use crate::ui::compositor::Compositor;
use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;

pub type ActionResult = Result<()>;

mod buffer;
mod command;
mod composite;
pub mod definition;
mod editing;
mod lsp;
mod mode;
mod movement;
mod search;
mod system;

pub use buffer::*;
pub use command::*;
pub use composite::*;
pub use definition::*;
pub use editing::*;
pub use lsp::*;
pub use mode::*;
pub use movement::*;
pub use search::*;
pub use system::*;

// Context passed to actions when they execute
pub struct ActionContext<'a> {
    pub buffer_manager: &'a mut BufferManager,
    pub command_buffer: &'a mut CommandBuffer,
    pub search_buffer: &'a mut SearchBuffer,
    pub message: &'a mut MessageManager,
    pub config: &'a Config,

    pub cursor: &'a mut Cursor,
    pub viewport: &'a mut Viewport,
    pub mode: &'a mut Mode,
    pub running: &'a mut bool,

    pub input_state: &'a mut InputState,

    pub compositor: &'a mut Compositor,
    pub component_ids: &'a ComponentIds,

    pub lsp_service: &'a mut LspService,
}

#[async_trait(?Send)]
pub trait Executable: Debug {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult;
}

// The Action trait defines what all actions must implement
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
macro_rules! impl_action {
    ($action_type:ty, $description:expr, $self:ident $definition_block:block) => {
        impl Action for $action_type {
            fn clone_box(&self) -> Box<dyn Action> {
                Box::new(self.clone())
            }

            fn describe(&self) -> &str {
                $description
            }

            fn to_serializable(&$self) -> ActionDefinition $definition_block
        }
    };

    ($action_type:ty, $description:expr, $definition:expr) => {
        impl Action for $action_type {
            fn clone_box(&self) -> Box<dyn Action> {
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

pub(super) use impl_action;
