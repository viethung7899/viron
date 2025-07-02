use crate::core::message::MessageType;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Style;
use crate::ui::{Bounds, Drawable, RenderContext};

pub struct MessageArea;

impl Drawable for MessageArea {
    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> anyhow::Result<()> {
        let Bounds {
            start_row, width, ..
        } = self.bounds(buffer, context);
        let Some(message) = context.message_manager.current_message() else {
            self.clear(buffer, context)?;
            return Ok(());
        };
        let formatted = format!("{:<width$}", message.content);
        let style = get_style_for_message(&message.message_type, context);
        buffer.set_text(start_row, 0, &formatted, &style);
        Ok(())
    }

    fn bounds(&self, buffer: &RenderBuffer, _context: &RenderContext) -> Bounds {
        Bounds {
            start_row: buffer.height - 1,
            start_col: 0,
            width: buffer.width,
            height: 1,
        }
    }
}

fn get_style_for_message(message_type: &MessageType, context: &RenderContext) -> Style {
    let mut style = context.theme.editor_style();
    let colors = &context.theme.colors.diagnostic;
    match message_type {
        MessageType::Error => {
            style.foreground = colors.error.foreground.clone();
        }
        _ => {}
    }
    style
}
