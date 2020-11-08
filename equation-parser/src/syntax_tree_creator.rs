use std::{clone::Clone, collections::HashSet, error::Error, fmt, fmt::Debug, hash::Hash};

#[derive(Debug, Eq, PartialEq)]
pub enum SyntaxTreeCreatorError {
    UnexpectedEOF,
    SymbolNotFound,
}

impl Error for SyntaxTreeCreatorError {}

impl fmt::Display for SyntaxTreeCreatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SyntaxTreeCreatorError::UnexpectedEOF => write!(f, "Unexpected End of File"),
            SyntaxTreeCreatorError::SymbolNotFound => write!(f, "Symbol Not Found"),
        }
    }
}

struct StateableIterator<'t, T> {
    values: &'t Vec<T>,
    index: usize,
}

impl<'t, T> StateableIterator<'t, T> {
    pub fn new(values: &'t Vec<T>) -> Self {
        Self { values, index: 0 }
    }

    pub fn get_state(&self) -> usize {
        self.index
    }
    pub fn set_state(&mut self, state: usize) {
        self.index = state;
    }
}

impl<'t, T> Iterator for StateableIterator<'t, T> {
    type Item = &'t T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.values.len() {
            self.index += 1;
            Some(&self.values[self.index - 1])
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub enum OptionalResults<SymbolEnum: Clone, TokenEnum: Clone> {
    Symbol(SymbolEnum),
    Token(TokenEnum),
}
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum OptionalSymbols<SymbolEnum: Clone + Hash, TokenEnum: Clone + Hash + Debug> {
    Symbol(SymbolEnum),
    Token(TokenEnum),
    EOF,
}

#[derive(Eq, PartialEq)]
enum OptionalInputs<TokenEnum: Eq> {
    Token(TokenEnum),
    EOF,
}

pub type PatternType<SymbolEnum, TokenEnum, UserResult> = (
    SymbolEnum,
    Vec<OptionalSymbols<SymbolEnum, TokenEnum>>,
    fn(Vec<UserResult>) -> Result<UserResult, Box<dyn Error>>,
);

pub struct SyntaxTreeCreator<SymbolEnum: Clone + Hash, TokenEnum: Clone + Hash + Debug, UserResult>
{
    patterns: Vec<PatternType<SymbolEnum, TokenEnum, UserResult>>,
    token_and_value_to_func_result: fn(TokenEnum, &str) -> UserResult,
}

impl<SymbolEnum: Eq + Clone + Hash, TokenEnum: Eq + Clone + Hash + Debug, UserResult>
    SyntaxTreeCreator<SymbolEnum, TokenEnum, UserResult>
{
    pub fn new(
        patterns: Vec<PatternType<SymbolEnum, TokenEnum, UserResult>>,
        token_and_value_to_func_result: fn(TokenEnum, &str) -> UserResult,
    ) -> Self {
        Self {
            patterns,
            token_and_value_to_func_result,
        }
    }

    fn filter_patterns_by_symbol(
        &self,
        symbol: &SymbolEnum,
    ) -> Vec<&PatternType<SymbolEnum, TokenEnum, UserResult>> {
        self.patterns
            .iter()
            .filter(|(s, _, _)| s == symbol)
            .collect()
    }

    fn match_pattern<'a>(
        &self,
        pattern: &Vec<OptionalSymbols<SymbolEnum, TokenEnum>>,
        tokens: &mut StateableIterator<(OptionalInputs<&TokenEnum>, &'a str)>,
        exclude_patterns: &mut HashSet<Vec<OptionalSymbols<SymbolEnum, TokenEnum>>>,
    ) -> Result<Vec<(OptionalResults<SymbolEnum, TokenEnum>, UserResult)>, Box<dyn Error>> {
        let mut values: Vec<(OptionalResults<SymbolEnum, TokenEnum>, UserResult)> = vec![];
        for (idx, symbol) in pattern.iter().enumerate() {
            match symbol {
                OptionalSymbols::Token(symbol) => {
                    let (token, word) = tokens
                        .next()
                        .ok_or_else(|| SyntaxTreeCreatorError::UnexpectedEOF)?;
                    match token {
                        OptionalInputs::Token(token) => {
                            if symbol != *token {
                                return Err(Box::new(SyntaxTreeCreatorError::SymbolNotFound));
                            }
                        }
                        OptionalInputs::EOF => {
                            return Err(Box::new(SyntaxTreeCreatorError::SymbolNotFound))
                        }
                    }
                    exclude_patterns.clear();
                    values.push((
                        OptionalResults::Token(symbol.clone()),
                        (self.token_and_value_to_func_result)(symbol.clone(), *word),
                    ));
                }
                OptionalSymbols::Symbol(symbol) => {
                    let mut exclude_patterns_temp: HashSet<_> =
                        exclude_patterns.iter().cloned().collect();

                    if idx == 0 {
                        exclude_patterns_temp.insert(pattern.clone());
                    }

                    let value = self.find_symbol(tokens, symbol, &mut exclude_patterns_temp)?;
                    values.push((OptionalResults::Symbol(symbol.clone()), value));
                }
                OptionalSymbols::EOF => {
                    exclude_patterns.clear();
                }
            }
        }
        return Ok(values);
    }

    fn filter_patterns_with_excluded_patterns<'a>(
        exclude_patterns: &HashSet<Vec<OptionalSymbols<SymbolEnum, TokenEnum>>>,
        patterns: &Vec<&'a PatternType<SymbolEnum, TokenEnum, UserResult>>,
    ) -> Vec<&'a PatternType<SymbolEnum, TokenEnum, UserResult>> {
        patterns
            .iter()
            .filter(|(_, x, _)| !exclude_patterns.contains(x))
            .cloned()
            .collect()
    }

    fn find_symbol(
        &self,
        tokens: &mut StateableIterator<(OptionalInputs<&TokenEnum>, &str)>,
        expected_symbol: &SymbolEnum,
        exclude_patterns: &mut HashSet<Vec<OptionalSymbols<SymbolEnum, TokenEnum>>>,
    ) -> Result<UserResult, Box<dyn Error>> {
        let patterns = self.filter_patterns_by_symbol(expected_symbol);
        let start_state = tokens.get_state();
        for (_, pattern, func) in
            Self::filter_patterns_with_excluded_patterns(&exclude_patterns, &patterns)
        {
            tokens.set_state(start_state);

            match self.match_pattern(&pattern, tokens, exclude_patterns) {
                Ok(values) => {
                    let values = values
                        .into_iter()
                        .map(|(_, x)| x)
                        .collect::<Vec<UserResult>>();
                    return Ok(func(values)?);
                }
                Err(e) => {
                    if let Some(syntax_tree_creator_error) =
                        e.downcast_ref::<SyntaxTreeCreatorError>()
                    {
                        if syntax_tree_creator_error != &SyntaxTreeCreatorError::SymbolNotFound {
                            return Err(e);
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        return Err(Box::new(SyntaxTreeCreatorError::SymbolNotFound));
    }

    pub fn analyze(
        &self,
        tokens: Vec<(&TokenEnum, &str)>,
        expected_symbol: &SymbolEnum,
    ) -> Result<UserResult, Box<dyn Error>> {
        let mut tokens = tokens
            .into_iter()
            .map(|(token, string)| (OptionalInputs::Token(token), string))
            .collect::<Vec<(OptionalInputs<&TokenEnum>, &str)>>();

        tokens.push((OptionalInputs::EOF, ""));
        let mut tokens = StateableIterator::new(&tokens);
        self.find_symbol(&mut tokens, &expected_symbol, &mut HashSet::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Eq, PartialEq, Hash, Clone)]
    enum SymbolEnum {
        Program,
        Value,
    }

    impl fmt::Display for SymbolEnum {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Debug::fmt(self, f)
        }
    }

    #[derive(Debug, Eq, PartialEq, Hash, Clone)]
    enum TokenEnum {
        And,
        Or,
        OpenBrackets,
        CloseBrackets,
        Not,
        Var,
        EOF,
    }
    impl fmt::Display for TokenEnum {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Debug::fmt(self, f)
        }
    }

    fn operator(items: Vec<String>) -> Result<String, Box<dyn Error>> {
        Ok(format!("({} {} {})", items[0], items[1], items[2]))
    }

    #[test]
    fn test_expected_tokens() {
        let patterns: Vec<PatternType<SymbolEnum, TokenEnum, String>> = {
            use OptionalSymbols::{Symbol, Token};
            vec![
                (
                    SymbolEnum::Program,
                    vec![Symbol(SymbolEnum::Value), Token(TokenEnum::EOF)],
                    |mut items| Ok(items.swap_remove(0)),
                ),
                (
                    SymbolEnum::Value,
                    vec![
                        Symbol(SymbolEnum::Value),
                        Token(TokenEnum::And),
                        Symbol(SymbolEnum::Value),
                    ],
                    operator,
                ),
                (
                    SymbolEnum::Value,
                    vec![
                        Symbol(SymbolEnum::Value),
                        Token(TokenEnum::Or),
                        Symbol(SymbolEnum::Value),
                    ],
                    operator,
                ),
                (
                    SymbolEnum::Value,
                    vec![
                        Token(TokenEnum::OpenBrackets),
                        Symbol(SymbolEnum::Value),
                        Token(TokenEnum::CloseBrackets),
                    ],
                    |items| Ok(items[1].clone()),
                ),
                (
                    SymbolEnum::Value,
                    vec![Token(TokenEnum::Not), Symbol(SymbolEnum::Value)],
                    |items| Ok(format!("not {}", items[1])),
                ),
                (
                    SymbolEnum::Value,
                    vec![Token(TokenEnum::Var)],
                    |mut items| Ok(items.swap_remove(0)),
                ),
            ]
        };
        let tokens = vec![
            (&TokenEnum::Var, "lala"),
            (&TokenEnum::And, "and"),
            (&TokenEnum::Var, "lolo"),
            (&TokenEnum::Or, "or"),
            (&TokenEnum::Var, "word"),
            (&TokenEnum::EOF, ""),
        ];

        let pattern_analyzer = SyntaxTreeCreator::new(patterns, |_, word| {
            let mut st = String::new();
            st.push_str(word);
            st
        });

        let value = pattern_analyzer
            .analyze(tokens, &SymbolEnum::Program)
            .unwrap();
        assert_eq!(value, String::from("(lala and (lolo or word))"))
    }
}
