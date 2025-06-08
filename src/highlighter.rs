use anyhow::Result;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};
use tree_sitter_rust::{HIGHLIGHTS_QUERY, LANGUAGE};

use crate::{editor::StyleInfo, theme::Theme};

pub struct Highlighter {
    parser: Parser,
    query: Query,
}

impl Highlighter {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = LANGUAGE.into();
        parser.set_language(&language)?;
        let query = Query::new(&language, HIGHLIGHTS_QUERY)?;
        Ok(Self { parser, query })
    }

    pub fn highlight(&mut self, code: &str, theme: &Theme) -> Result<Vec<StyleInfo>> {
        let tree = self.parser.parse(code, None).expect("Failed to parse code");
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), code.as_bytes());
        let mut colors = Vec::new();

        while let Some(matching) = matches.next() {
            for capture in matching.captures {
                let node = capture.node;
                let range = node.byte_range();

                let scope = self.query.capture_names()[capture.index as usize];
                let style = theme.get_style(scope);
                // let content = &code[range.clone()];

                if let Some(style) = style {
                    // log!("[found] {scope} = {content}");
                    colors.push(StyleInfo { range, style });
                } else {
                    // log!("[not found] {scope} = {content}");
                }
            }
        }
        Ok(colors)
    }
}
