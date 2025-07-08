use anyhow::{anyhow, Result};
use std::ops::Range;
use tree_sitter::{Parser, Point, Query, QueryCursor, StreamingIterator, Tree};

use crate::core::history::edit::Edit;
use crate::core::language::Language;

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub byte_range: Range<usize>,
    pub start_position: Point,
    pub end_position: Point,
    pub scope: String,
}

pub struct SyntaxEngine {
    parser: Parser,
    query: Query,
    tree: Option<Tree>,
}

impl SyntaxEngine {
    pub fn new(language: &Language) -> Result<Self> {
        let Some(ts_language) = language.get_tree_sitter_language() else {
            return Err(anyhow!(
                "{} does not have a Tree-sitter language defined",
                language.to_str()
            ));
        };
        let Some(query_src) = language.get_highlight_query() else {
            return Err(anyhow!(
                "{} does not have a Tree-sitter query defined",
                language.to_str()
            ));
        };

        let mut parser = Parser::new();
        parser.set_language(&ts_language)?;
        let query = Query::new(&ts_language, query_src)?;

        Ok(Self {
            parser,
            query,
            tree: None,
        })
    }

    pub fn apply_edit(&mut self, edit: &Edit) -> Result<()> {
        let Some(tree) = &mut self.tree else {
            return Ok(());
        };
        match edit {
            Edit::Insert(insert) => {
                tree.edit(&insert.edit_summary());
            }
            Edit::Delete(delete) => {
                tree.edit(&delete.edit_summary());
            }
        };
        Ok(())
    }

    pub fn highlight(&mut self, code: &[u8]) -> Result<Vec<TokenInfo>> {
        let mut tokens = Vec::new();
        self.tree = self.parser.parse(code, self.tree.as_ref());
        let Some(tree) = &self.tree else {
            return Ok(tokens);
        };

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), code);

        while let Some(matching) = matches.next() {
            for capture in matching.captures {
                let node = capture.node;
                let scope = self.query.capture_names()[capture.index as usize];

                tokens.push(TokenInfo {
                    byte_range: node.byte_range(),
                    start_position: node.start_position(),
                    end_position: node.end_position(),
                    scope: scope.to_string(),
                });
            }
        }
        Ok(tokens)
    }
}
