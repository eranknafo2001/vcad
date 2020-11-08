extern crate regex;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum TokenizerError {
    NoMatchError(String),
}

impl Error for TokenizerError {}

impl fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenizerError::NoMatchError(s) => write!(f, "No Match Pattern was found {}", s),
        }
    }
}

pub struct Tokenizer<TokenEnum: Clone> {
    regex_patterns: Vec<(TokenEnum, regex::Regex)>,
    white_spaces_chars: Vec<regex::Regex>,
}

impl<TokenEnum: Clone> Tokenizer<TokenEnum> {
    pub fn new(
        tokens: Vec<(TokenEnum, String)>,
        white_spaces_chars: Vec<&str>,
    ) -> Result<Self, Box<dyn Error>> {
        let regex_patterns = tokens
            .iter()
            .map(|(name, pattern)| {
                Ok((
                    name.clone(),
                    regex::Regex::new(format!("^{}", pattern).as_str())?,
                ))
            })
            .collect::<Result<Vec<(TokenEnum, regex::Regex)>, regex::Error>>()?;

        let white_spaces_chars = white_spaces_chars
            .iter()
            .map(|pattern| regex::Regex::new(format!("^{}", pattern).as_str()))
            .collect::<Result<Vec<regex::Regex>, regex::Error>>()?;

        Ok(Self {
            regex_patterns,
            white_spaces_chars,
        })
    }

    fn match_token<'s, 't>(
        &'s self,
        text: &'t str,
    ) -> Result<(&'s TokenEnum, &'t str, &'t str), Box<dyn Error>> {
        for (name, regex_pattern) in self.regex_patterns.iter() {
            if let Some(match_pattern) = regex_pattern.find(text) {
                return Ok((name, match_pattern.as_str(), &text[match_pattern.end()..]));
            }
        }
        let mut err_msg = String::new();
        err_msg.push_str(&text[0..10.min(text.len())]);
        Err(Box::new(TokenizerError::NoMatchError(err_msg)))
    }

    fn clear_white_spaces<'t>(&self, text: &'t str) -> &'t str {
        let mut text = text;
        loop {
            let mut found = false;
            for regex_pattern in self.white_spaces_chars.iter() {
                if let Some(match_pattern) = regex_pattern.find(text) {
                    text = &text[match_pattern.end()..];
                    found = true;
                    break;
                }
            }
            if !found {
                return text;
            }
        }
    }

    pub fn tokenize<'s, 't>(
        &'s self,
        mut text: &'t str,
    ) -> Result<Vec<(&'s TokenEnum, &'t str)>, Box<dyn Error>> {
        let mut result = vec![];
        loop {
            if text.is_empty() {
                return Ok(result);
            }
            text = self.clear_white_spaces(text);
            let (name, word, res_text) = self.match_token(text)?;
            text = res_text;
            result.push((name, word));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Eq, PartialEq, Hash, Debug)]
    enum Items {
        OpenBrackets,
        CloseBrackets,
        And,
        Or,
        Not,
        Lala,
        Var,
    }

    #[test]
    fn test_good_tokens() {
        let tokens = vec![
            (Items::OpenBrackets, String::from("\\(")),
            (Items::CloseBrackets, String::from("\\)")),
            (Items::And, String::from("and")),
            (Items::Or, String::from("or")),
            (Items::Not, String::from("not")),
            (Items::Lala, String::from("lala")),
            (Items::Var, String::from("[a-z|A-Z][a-z|A-Z|0-9]*")),
        ];
        let white_spaces = vec!["\n", " ", "\t"];
        let tokenizer = Tokenizer::new(tokens, white_spaces).unwrap();
        let text_test = "(lala and row) or lala";

        let tokens = tokenizer.tokenize(text_test).unwrap();

        assert_eq!(tokens[0].0, &Items::OpenBrackets);
        assert_eq!(tokens[1].0, &Items::Lala);
        assert_eq!(tokens[2].0, &Items::And);
        assert_eq!(tokens[3].0, &Items::Var);
        assert_eq!(tokens[4].0, &Items::CloseBrackets);
        assert_eq!(tokens[5].0, &Items::Or);
        assert_eq!(tokens[6].0, &Items::Lala);
    }

    #[test]
    fn test_invalid_tokens() {
        let tokens = vec![
            (Items::OpenBrackets, String::from("\\(")),
            (Items::CloseBrackets, String::from("\\)")),
            (Items::And, String::from("and")),
            (Items::Or, String::from("or")),
            (Items::Not, String::from("not")),
            (Items::Lala, String::from("lala")),
            (Items::Var, String::from("[a-z|A-Z][a-z|A-Z|0-9]*")),
        ];
        let white_spaces = vec!["\n", " ", "\t"];
        let tokenizer = Tokenizer::new(tokens, white_spaces).unwrap();
        let text_test = "lala and 43dsdsf p42 2dsds-fegfd";

        assert!(tokenizer.tokenize(text_test).is_err())
    }
}
