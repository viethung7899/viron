use crate::ui::components::{CommandLine, EditorView, MessageArea, PendingKeys, SearchBox, StatusLine};
use crate::ui::compositor::Compositor;
use anyhow::Result;
use crate::constants::components::{COMMAND_LINE, EDITOR_VIEW, MESSAGE_AREA, PENDING_KEYS, SEARCH_BOX, STATUS_LINE};

pub struct UISystem {
    pub compositor: Compositor,
}

impl UISystem {
    pub fn new(width: usize, height: usize) -> Result<Self> {
        let mut compositor = Compositor::new(width, height);

        // Add components to the compositor
        compositor.add_component(STATUS_LINE, StatusLine, true)?;
        compositor.add_focusable_component(EDITOR_VIEW, EditorView::new(), true)?;
        compositor.set_focus(EDITOR_VIEW)?;

        // Add invisible components
        compositor.add_component(PENDING_KEYS, PendingKeys, false)?;
        compositor.add_focusable_component(COMMAND_LINE, CommandLine, false)?;
        compositor.add_focusable_component(SEARCH_BOX, SearchBox, false)?;
        compositor.add_component(MESSAGE_AREA, MessageArea, false)?;


        Ok(Self {
            compositor,
        })
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.compositor.resize(width, height);
    }

    pub fn mark_all_dirty(&mut self) {
        self.compositor.mark_all_dirty()
    }

    pub fn mark_dirty<const N: usize>(&mut self, ids: [&str; N]) -> Result<()> {
        for id in ids {
            self.compositor.mark_dirty(id)?;
        }
        Ok(())
    }
}