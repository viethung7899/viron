use crate::ui::components::Component;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Style;
use crate::ui::RenderContext;
use anyhow::{anyhow, Result};
use std::io::Write;

pub struct Compositor {
    components: Vec<Component>,
    editor_style: Style,
    current_buffer: RenderBuffer,
    previous_buffer: Option<RenderBuffer>,
}

impl Compositor {
    pub fn new(width: usize, height: usize, default_style: &Style) -> Self {
        Self {
            components: Vec::new(),
            editor_style: default_style.clone(),
            current_buffer: RenderBuffer::new(width, height, default_style),
            previous_buffer: None,
        }
    }

    pub fn add_component(&mut self, component: Component) -> Result<String> {
        let id = component.id.clone();
        if self.components.iter().any(|c| c.id == id) {
            return Err(anyhow!("Component already exists"));
        }
        self.components.push(component);
        Ok(id)
    }

    pub fn remove_component(&mut self, component_id: &str) {
        self.components.retain(|c| c.id != component_id);
    }

    pub fn mark_dirty(&mut self, component_id: &str) -> Result<()> {
        if let Some(component) = self.components.iter_mut().find(|c| c.id == component_id) {
            component.dirty = true;
            Ok(())
        } else {
            Err(anyhow!("Component not found"))
        }
    }

    pub fn mark_visible(&mut self, component_id: &str, visible: bool) -> Result<()> {
        if let Some(component) = self.components.iter_mut().find(|c| c.id == component_id) {
            if component.visible == visible {
                return Ok(());
            }
            component.visible = visible;
            component.dirty = true;
            Ok(())
        } else {
            Err(anyhow!("Component not found"))
        }
    }

    pub fn mark_all_dirty(&mut self) {
        for component in self.components.iter_mut() {
            component.dirty = true;
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.current_buffer = RenderBuffer::new(width, height, &self.editor_style);
        // Invalidate previous buffer on resize
        self.previous_buffer = None;
        self.mark_all_dirty();
    }

    // Render using diff
    pub fn render<W: Write>(&mut self, context: &mut RenderContext, writer: &mut W) -> Result<()> {
        // Render all dirty components to the current buffer
        for component in self.components.iter_mut().filter(|c| c.dirty) {
            if component.visible {
                component.drawable.draw(&mut self.current_buffer, context)?;
            } else {
                component
                    .drawable
                    .clear(&mut self.current_buffer, context)?;
            }
            component.dirty = false; // Clear dirty flag after rendering
        }

        // If we have a previous buffer, do differential rendering
        if let Some(ref previous) = self.previous_buffer {
            for change in self.current_buffer.diff(previous) {
                change.flush(writer, &self.editor_style)?
            }
        } else {
            // No previous buffer, do full render
            self.current_buffer.flush(writer)?;
        }

        // Store current buffer as previous for next diff
        self.previous_buffer = Some(self.current_buffer.clone());

        Ok(())
    }

    // Force a full re-render (useful after resize or major changes)
    pub fn invalidate(&mut self) {
        self.previous_buffer = None;
    }
}
