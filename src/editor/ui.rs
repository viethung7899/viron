use crate::ui::components::{CommandLine, ComponentIds, EditorView, MessageArea, PendingKeys, SearchBox, StatusLine};
use crate::ui::compositor::Compositor;
use anyhow::Result;

pub struct UISystem {
    pub compositor: Compositor,
    pub component_ids: ComponentIds,
}

impl UISystem {
    pub fn new(width: usize, height: usize) -> Result<Self> {
        let mut compositor = Compositor::new(width, height);

        // Add components to the compositor
        let status_line_id = compositor.add_component("status_line", StatusLine, true)?;
        let editor_view_id =
            compositor.add_focusable_component("editor_view", EditorView::new(), true)?;

        // Add invisible components
        let pending_keys_id = compositor.add_component("pending_keys", PendingKeys, false)?;
        let command_line_id =
            compositor.add_focusable_component("command_line", CommandLine, false)?;
        let search_box_id = compositor.add_focusable_component("search_box", SearchBox, false)?;
        let message_area_id = compositor.add_component("message_area", MessageArea, false)?;

        compositor.set_focus(&editor_view_id)?;

        let component_ids = ComponentIds {
            status_line_id,
            editor_view_id,
            pending_keys_id,
            command_line_id,
            message_area_id,
            search_box_id,
        };

        Ok(Self {
            compositor,
            component_ids,
        })
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.compositor.resize(width, height);
    }

    pub fn mark_editor_dirty(&mut self) -> Result<()> {
        self.compositor.mark_dirty(&self.component_ids.editor_view_id)
    }

    pub fn mark_status_dirty(&mut self) -> Result<()> {
        self.compositor.mark_dirty(&self.component_ids.status_line_id)
    }
}