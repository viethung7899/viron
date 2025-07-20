use crate::ui::components::Component;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::{Drawable, Focusable};
use anyhow::{anyhow, Result};
use std::rc::Rc;
use std::{collections::HashMap, io::Write};
use crate::ui::context::RenderContext;

pub struct Compositor {
    components: HashMap<String, Component>,
    current_buffer: RenderBuffer,
    previous_buffer: Option<RenderBuffer>,
    focused_component: Option<String>,
}

impl Compositor {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            components: HashMap::new(),
            current_buffer: RenderBuffer::new(width, height),
            previous_buffer: None,
            focused_component: None,
        }
    }

    pub fn add_component<C: Drawable + 'static>(
        &mut self,
        id: &str,
        drawable: C,
        visible: bool,
    ) -> Result<String> {
        let component = Component {
            dirty: true,
            visible,
            drawable: Rc::new(drawable),
            focusable: None,
        };
        self.add_internal_component(id, component)
    }

    pub fn add_focusable_component<C: Drawable + Focusable + 'static>(
        &mut self,
        id: &str,
        drawable: C,
        visible: bool,
    ) -> Result<String> {
        let drawable = Rc::new(drawable);
        let focusable = drawable.clone();

        let component = Component {
            dirty: true,
            visible,
            drawable,
            focusable: Some(focusable),
        };
        self.add_internal_component(id, component)
    }

    fn add_internal_component(&mut self, id: &str, component: Component) -> Result<String> {
        if self.components.contains_key(id) {
            return Err(anyhow!("Component already exists"));
        }
        self.components.insert(id.to_string(), component);
        Ok(id.to_string())
    }

    pub fn remove_component(&mut self, component_id: &str) {
        self.components.remove(component_id);
    }

    pub fn get_component_mut(&mut self, component_id: &str) -> Option<&mut Component> {
        self.components.get_mut(component_id)
    }

    pub fn mark_dirty(&mut self, component_id: &str) -> Result<()> {
        if let Some(component) = self.components.get_mut(component_id) {
            component.dirty = true;
            Ok(())
        } else {
            Err(anyhow!("Component not found"))
        }
    }

    pub fn mark_visible(&mut self, component_id: &str, visible: bool) -> Result<()> {
        if let Some(component) = self.components.get_mut(component_id) {
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
        for component in self.components.values_mut() {
            component.dirty = true;
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.current_buffer = RenderBuffer::new(width, height);
        // Invalidate previous buffer on resize
        self.previous_buffer = None;
        self.mark_all_dirty();
    }

    pub fn set_focus(&mut self, component_id: &str) -> Result<()> {
        if let Some(component) = self.components.get(component_id) {
            if component.focusable.is_some() {
                self.focused_component = Some(component_id.to_string());
                Ok(())
            } else {
                Err(anyhow!("Component is not focusable"))
            }
        } else {
            Err(anyhow!("Component not found"))
        }
    }

    // Render using diff
    pub fn render<'a, W: Write>(
        &mut self,
        context: &mut RenderContext<'a>,
        writer: &mut W,
    ) -> Result<()> {
        // Render all dirty components to the current buffer
        for component in self.components.values_mut().filter(|c| c.dirty) {
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
        let editor_style = context.config.theme.editor_style();
        if let Some(ref previous) = self.previous_buffer {
            for change in self.current_buffer.diff(previous) {
                change.flush(writer, &editor_style)?
            }
        } else {
            // No previous buffer, do full render
            self.current_buffer.flush(writer, &editor_style)?;
        }

        // Store current buffer as previous for next diff
        self.previous_buffer = Some(self.current_buffer.clone());

        Ok(())
    }

    pub fn get_cursor_position<'a>(&self, context: &RenderContext<'a>) -> Option<(usize, usize)> {
        let focused_id = self.focused_component.as_ref()?;
        let component = self.components.get(focused_id)?;
        let focusable = component.focusable.as_ref()?;
        Some(focusable.get_display_cursor(&self.current_buffer, context))
    }

    // Force a full re-render (useful after resize or major changes)
    pub fn invalidate(&mut self) {
        self.previous_buffer = None;
    }
}
