use crate::lexer::{Operator, Token};
use anyhow::{Result, bail};

pub fn parse_tokens(s: Vec<Token>) -> Result<ShellCmd> {
    assert_ne!(
        s,
        [],
        "parse_tokens should never be called with an empty list of tokens"
    );
    let mut l = Parser::new(s);
    l.parse_to_cmd()
}

#[derive(Debug, PartialEq)]
pub struct ShellCmd {
    pub name: String,
    pub args: Vec<String>,
    pub stdout_dest: OutputType,
    pub stderr_dest: OutputType,
}

#[derive(Debug, PartialEq)]
pub enum OutputType {
    Inherit,
    OverwriteFile(String),
    AppendFile(String),
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(s: Vec<Token>) -> Self {
        Self { tokens: s, pos: 0 }
    }

    fn parse_to_cmd(&mut self) -> Result<ShellCmd> {
        let Some(name) = self.word() else {
            bail!("Malformed input: expected word to be first token")
        };
        let mut args = vec![];
        while let Some(s) = self.word() {
            args.push(s);
        }

        let mut stdout_dest = OutputType::Inherit;
        let mut stderr_dest = OutputType::Inherit;
        if let Some(op) = self.operator() {
            let Some(path) = self.word() else {
                bail!("Malformed input: expected token after operator")
            };
            // QUESTION: n2  is this idiomatic?
            match op {
                Operator::RedirectStdout => stdout_dest = OutputType::OverwriteFile(path),
                Operator::RedirectStderr => stderr_dest = OutputType::OverwriteFile(path),
                Operator::AppendStdout => stdout_dest = OutputType::AppendFile(path),
                Operator::AppendStderr => stderr_dest = OutputType::AppendFile(path),
            }
        }

        if self.peek().is_some() {
            bail!("Malformed input: unexpected token after redirection")
        }

        Ok(ShellCmd {
            name,
            args,
            stdout_dest,
            stderr_dest,
        })
    }

    /// Try to consume a word from `tokens`, returning `None` otherwise.
    fn word(&mut self) -> Option<String> {
        match self.peek() {
            Some(Token::Word(s)) => {
                let s = s.clone();
                self.bump();
                Some(s)
            }
            Some(Token::Operator(_)) | None => None,
        }
    }

    /// Try to consume an operator from `tokens`, returning `None` otherwise.
    fn operator(&mut self) -> Option<Operator> {
        match self.peek() {
            Some(Token::Operator(op)) => {
                let op = *op;
                self.bump();
                Some(op)
            }
            Some(Token::Word(_)) | None => None,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn bump(&mut self) -> Option<&Token> {
        let c = self.tokens.get(self.pos)?;
        self.pos += 1;
        Some(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::split_tokens;

    fn file(path: &str) -> OutputType {
        OutputType::OverwriteFile(path.to_string())
    }

    /// Shorthand for an expected `ShellCmd`.
    fn cmd(
        name: &str,
        args: &[&str],
        stdout_dest: OutputType,
        stderr_dest: OutputType,
    ) -> ShellCmd {
        ShellCmd {
            name: name.to_string(),
            args: args.iter().map(|a| a.to_string()).collect(),
            stdout_dest,
            stderr_dest,
        }
    }

    #[test]
    fn test_parse_ok() {
        // (tokens, expected command)
        let cases: Vec<(Vec<Token>, ShellCmd)> = vec![
            (
                split_tokens("echo foo > /dev/null").unwrap(),
                cmd("echo", &["foo"], file("/dev/null"), OutputType::Inherit),
            ),
            (
                split_tokens("fish --help").unwrap(),
                cmd(
                    "fish",
                    &["--help"],
                    OutputType::Inherit,
                    OutputType::Inherit,
                ),
            ),
        ];
        for (tokens, expected) in cases {
            let input = format!("{tokens:?}");
            let actual = parse_tokens(tokens).unwrap_or_else(|e| panic!("input: {input}: {e}"));
            assert_eq!(actual, expected, "input: {input}");
        }
    }

    #[test]
    fn test_parse_err() {
        // Token lists that must be rejected. Must be non-empty: main filters out empty lines
        // before the parser runs, and the parser asserts on it.
        let cases: Vec<Vec<Token>> = vec![
            split_tokens("echo foo 2>").unwrap(),
            split_tokens("echo bar >>").unwrap(),
        ];
        for tokens in cases {
            let input = format!("{tokens:?}");
            assert!(parse_tokens(tokens).is_err(), "input: {input}");
        }
    }
}
