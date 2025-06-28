mod language;

use std::ops::Range;

use anyhow::{Ok, Result};
use tree_sitter::{Parser, Point, Query, QueryCursor, StreamingIterator};
use tree_sitter_rust::{HIGHLIGHTS_QUERY, LANGUAGE};

use crate::core::syntax::language::LanguageType;

pub struct Highlighter {
    parser: Parser,
    language: LanguageType,
    query: Option<Query>,
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub byte_range: Range<usize>,
    pub start_position: Point,
    pub end_position: Point,
    pub scope: String,
}

impl Highlighter {
    pub fn new(language: LanguageType) -> Result<Self> {
        let mut highlighter = Self {
            parser: Parser::new(),
            language,
            query: None,
        };

        if let Some((ts_language, highlight_query)) = language
            .get_tree_sitter_language()
            .zip(language.get_highlight_query())
        {
            highlighter.parser.set_language(&ts_language)?;
            highlighter.query = Some(Query::new(&ts_language, HIGHLIGHTS_QUERY)?);
        }
        Ok(highlighter)
    }

    pub fn highlight(&mut self, code: &[u8]) -> Result<Vec<TokenInfo>> {
        let Some(query) = &self.query else {
            return Ok(vec![]);
        };
        let tree = self.parser.parse(code, None);
        let mut colors = Vec::new();
        let Some(tree) = tree else {
            return Ok(colors);
        };
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), code);

        while let Some(matching) = matches.next() {
            for capture in matching.captures {
                let node = capture.node;
                let scope = query.capture_names()[capture.index as usize];

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
