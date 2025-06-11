use std::ops::Range;

use anyhow::Result;
use tree_sitter::{Parser, Point, Query, QueryCursor, StreamingIterator, Tree};
use tree_sitter_rust::{HIGHLIGHTS_QUERY, LANGUAGE};

pub struct Highlighter {
    parser: Parser,
    query: Query,
    tree: Option<Tree>,
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub byte_range: Range<usize>,
    pub start_position: Point,
    pub end_position: Point,
    pub scope: String,
}

impl Highlighter {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = LANGUAGE.into();
        parser.set_language(&language)?;
        let query = Query::new(&language, HIGHLIGHTS_QUERY)?;
        Ok(Self {
            parser,
            query,
            tree: None,
        })
    }

    pub fn highlight(&mut self, code: &[u8]) -> Result<Vec<TokenInfo>> {
        self.tree = self.parser.parse(code, self.tree.as_ref());
        let mut colors = Vec::new();
        let Some(tree) = &self.tree else {
            return Ok(colors);
        };
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), code);

        while let Some(matching) = matches.next() {
            for capture in matching.captures {
                let node = capture.node;
                let scope = self.query.capture_names()[capture.index as usize];

                colors.push(TokenInfo {
                    byte_range: node.byte_range(),
                    start_position: node.start_position(),
                    end_position: node.end_position(),
                    scope: scope.to_string(),
                });
            }
        }
        Ok(colors)
    }
}
