use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Style;
use crate::ui::{Drawable, RenderContext};
use anyhow::{Result, anyhow};
use std::collections::HashSet;
use std::hash::Hash;
use std::io::Write;

type Component<'a> = Box<dyn Drawable + 'a>;

impl PartialEq for Component<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Component<'_> {}
impl Hash for Component<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

pub struct Compositor<'a> {
    components: HashSet<Component<'a>>,
    dirty_set: HashSet<String>,
    editor_style: Style,
    current_buffer: RenderBuffer,
    previous_buffer: Option<RenderBuffer>,
}

impl<'a> Compositor<'a> {
    pub fn new(width: usize, height: usize, default_style: &Style) -> Self {
        Self {
            components: HashSet::new(),
            dirty_set: HashSet::new(),
            editor_style: default_style.clone(),
            current_buffer: RenderBuffer::new(width, height, default_style),
            previous_buffer: None,
        }
    }

    pub fn add_component(&mut self, component: Box<dyn Drawable + 'a>) -> Result<String> {
        let id = component.id().to_string();

        if self.components.contains(&component) {
            return Err(anyhow!("Component already exists"));
        }

        self.components.insert(component);
        self.dirty_set.insert(id.clone());
        Ok(id)
    }

    pub fn remove_component(&mut self, component_id: &str) {
        self.components.retain(|c| c.id() != component_id);
        self.dirty_set.remove(component_id);
    }

    pub fn mark_dirty(&mut self, component_id: &str) -> Result<()> {
        if (self.components.iter().any(|c| c.id() == component_id)) {
            self.dirty_set.insert(component_id.to_string());
            Ok(())
        } else {
            Err(anyhow!("Component does not exist"))
        }
    }

    pub fn mark_all_dirty(&mut self) {
        for component in self.components.iter() {
            self.dirty_set.insert(component.id().to_string());
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.current_buffer = RenderBuffer::new(width, height, &self.editor_style);
        // Invalidate previous buffer on resize
        self.previous_buffer = None;
        self.mark_all_dirty();
    }

    // Render using diff
    pub fn render<W: Write>(&mut self, context: &RenderContext, writer: &mut W) -> Result<()> {
        // Render all components to the current buffer
        let components = self
            .components
            .iter()
            .filter(|&component| self.dirty_set.contains(component.id()));

        for component in components {
            component.draw(&mut self.current_buffer, context);
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
        writer.flush()?;
        self.previous_buffer = Some(self.current_buffer.clone());
        self.dirty_set.clear();

        Ok(())
    }

    // Force a full re-render (useful after resize or major changes)
    pub fn invalidate(&mut self) {
        self.previous_buffer = None;
    }
}
