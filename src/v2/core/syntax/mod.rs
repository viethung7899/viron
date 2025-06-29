mod language;

use std::{collections::HashMap, ops::Range};

use anyhow::{Ok, Result};
use tree_sitter::{Parser, Point, Query, QueryCursor, StreamingIterator};

pub use language::LanguageType;

pub struct SyntaxHighlighter {
    parser: Parser,
    language: Option<LanguageType>,
    queries: HashMap<LanguageType, Query>,
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub byte_range: Range<usize>,
    pub start_position: Point,
    pub end_position: Point,
    pub scope: String,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            language: None,
            queries: HashMap::new(),
        }
    }

    pub fn set_langauge(&mut self, language: LanguageType) -> Result<()> {
        if self.language == Some(language) {
            return Ok(());
        }

        let Some(ts_language) = language.get_tree_sitter_language() else {
            self.parser.reset();
            return Ok(());
        };

        if !self.queries.contains_key(&language) {
            if let Some(source) = language.get_highlight_query() {
                let query = Query::new(&ts_language, source)?;
                self.queries.insert(language, query);
            };
        }

        self.language = Some(language);

        Ok(())
    }

    pub fn clear_cache(&mut self) {
        self.parser.reset();
    }

    pub fn highlight(&mut self, code: &[u8]) -> Result<Vec<TokenInfo>> {
        let mut colors = Vec::new();

        let Some(language) = self.language else {
            return Ok(colors);
        };

        let Some(query) = self.queries.get(&language) else {
            return Ok(colors);
        };

        let Some(tree) = self.parser.parse(code, None) else {
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
