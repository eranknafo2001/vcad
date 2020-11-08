use std::{clone::Clone, cmp::Eq, error::Error, fmt::Debug, hash::Hash};

use crate::syntax_tree_creator::{PatternType, SyntaxTreeCreator};
use crate::tokenizer::Tokenizer;

pub struct Compiler<TokenEnum: Clone + Hash + Debug, SymbolEnum: Clone + Hash, UserResult> {
    tokenizer: Tokenizer<TokenEnum>,
    pattern_analyzer: SyntaxTreeCreator<SymbolEnum, TokenEnum, UserResult>,
    program_token: SymbolEnum,
}

impl<TokenEnum: Clone + Hash + Eq + Debug, SymbolEnum: Clone + Hash + Eq, UserResult>
    Compiler<TokenEnum, SymbolEnum, UserResult>
{
    pub fn new(
        tokens: Vec<(TokenEnum, String)>,
        white_spaces_chars: Vec<&str>,
        patterns: Vec<PatternType<SymbolEnum, TokenEnum, UserResult>>,
        program_token: SymbolEnum,
        token_and_value_to_func_result: fn(TokenEnum, &str) -> UserResult,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            tokenizer: Tokenizer::new(tokens, white_spaces_chars)?,
            pattern_analyzer: SyntaxTreeCreator::new(patterns, token_and_value_to_func_result),
            program_token,
        })
    }

    pub fn compile(&self, text: &str) -> Result<UserResult, Box<dyn Error>> {
        let tokens = self.tokenizer.tokenize(text)?;
        Ok(self.pattern_analyzer.analyze(tokens, &self.program_token)?)
    }
}
