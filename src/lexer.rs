use anyhow::{Result, bail};

pub fn split_tokens(s: &str) -> Result<Vec<Token>> {
    let mut l = Lexer::new(s);
    l.split_tokens()
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Word(String),
    Operator(Operator),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operator {
    RedirectStdout,
    RedirectStderr,
    AppendStdout,
    AppendStderr,
}

struct Lexer {
    text: String,
    pos: usize,
}

impl Lexer {
    fn new(s: &str) -> Self {
        Self {
            text: s.to_string(),
            pos: 0,
        }
    }

    // input ::= skip (token skip)*
    fn split_tokens(&mut self) -> Result<Vec<Token>> {
        self.skip();
        let mut result = vec![];
        while self.peek().is_some() {
            let w = self.token()?;
            result.push(w);
            self.skip();
        }
        Ok(result)
    }

    // skip ::= \s*
    fn skip(&mut self) {
        while let Some(c) = self.peek()
            && c.is_ascii_whitespace()
        {
            self.bump();
        }
    }

    // Note: in my shell grammar, I am deciding not to support using operator as a suffix.
    // E.g. I do not support `echo foo>/dev/null`. That must be written as `echo foo > /dev/null`.
    //
    // token ::= (single_quoted | double_quoted | unquoted)+
    fn token(&mut self) -> Result<Token> {
        let start = self.pos;
        let mut buf = String::new();
        while let Some(c) = self.peek() {
            match c {
                '\'' => self.single_quoted(&mut buf)?,
                '"' => self.double_quoted(&mut buf)?,
                _ if c.is_ascii_whitespace() => break,
                _ => self.unquoted(&mut buf)?,
            }
        }
        let span: &str = &self.text[start..self.pos];
        let token = match span {
            ">" | "1>" => Token::Operator(Operator::RedirectStdout),
            "2>" => Token::Operator(Operator::RedirectStderr),
            ">>" | "1>>" => Token::Operator(Operator::AppendStdout),
            "2>>" => Token::Operator(Operator::AppendStderr),
            _ => Token::Word(buf),
        };
        Ok(token)
    }

    // unquoted ::= ([^'"\s\\] | \\ .)+
    fn unquoted(&mut self, buf: &mut String) -> Result<()> {
        while let Some(c) = self.peek()
            && !c.is_ascii_whitespace()
            && !matches!(c, '\'' | '"')
        {
            if c == '\\' {
                let next = self.escape()?;
                buf.push(next);
            } else {
                buf.push(c);
                self.bump();
            }
        }
        Ok(())
    }

    // single_quoted ::= "'" [^']* "'"
    fn single_quoted(&mut self, buf: &mut String) -> Result<()> {
        assert_eq!(self.peek(), Some('\''));
        self.bump();
        while let Some(c) = self.peek()
            && c != '\''
        {
            buf.push(c);
            self.bump();
        }
        let closing_quote = self.bump();
        if !matches!(closing_quote, Some('\'')) {
            bail!("Malformed input")
        }
        Ok(())
    }

    // double_quoted ::= '"' ([^"\\] | \\ .)* '"'
    fn double_quoted(&mut self, buf: &mut String) -> Result<()> {
        assert_eq!(self.peek(), Some('"'));
        self.bump();
        while let Some(c) = self.peek()
            && c != '"'
        {
            if c == '\\' {
                let next = self.escape()?;
                if !matches!(next, '"' | '\\' | '$' | '`') {
                    buf.push('\\');
                }
                buf.push(next);
            } else {
                buf.push(c);
                self.bump();
            }
        }
        let closing_quote = self.bump();
        if !matches!(closing_quote, Some('"')) {
            bail!("Malformed input")
        }
        Ok(())
    }

    fn escape(&mut self) -> Result<char> {
        assert_eq!(self.peek(), Some('\\'));
        self.bump();
        let Some(next) = self.bump() else {
            bail!("Malformed input: expecting character after backslash");
        };
        Ok(next)
    }

    fn peek(&self) -> Option<char> {
        self.text[self.pos..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lexes `s`, expecting only words; panics on an operator.
    fn words(s: &str) -> Result<Vec<String>> {
        Ok(split_tokens(s)?
            .into_iter()
            .map(|t| match t {
                Token::Word(w) => w,
                Token::Operator(_) => panic!("unexpected operator in {s:?}"),
            })
            .collect())
    }

    #[test]
    fn test_token_splitting_unquoted() {
        assert_eq!(words("foo").unwrap(), ["foo"]);
        assert_eq!(words("echo hi").unwrap(), ["echo", "hi"]);
        assert_eq!(words(" john\tdoe  ").unwrap(), ["john", "doe"]);
        assert_eq!(words(r"foo\ ").unwrap(), ["foo "]);
        assert_eq!(words(r"foo\ bar").unwrap(), ["foo bar"]);
        assert_eq!(words(r"foo\   bar").unwrap(), ["foo ", "bar"]);
        assert_eq!(words(r#"foo\""#).unwrap(), ["foo\""]);
        // Outside quotes a backslash always disappears, whatever follows it.
        assert_eq!(words(r"a\b").unwrap(), ["ab"]);
        assert_eq!(words(r"\\").unwrap(), ["\\"]);
        assert_eq!(words(r"\'").unwrap(), ["'"]);
    }

    #[test]
    fn test_token_splitting_single_quoted() {
        assert_eq!(words("foo 'bar'").unwrap(), ["foo", "bar"]);
        assert_eq!(words("'a'b").unwrap(), ["ab"]);
        assert_eq!(words("'a   b'").unwrap(), ["a   b"]);
        assert_eq!(words("''").unwrap(), [""]);
        // Backslash is literal inside single quotes, even before the closing quote.
        assert_eq!(words(r"'a\b'").unwrap(), [r"a\b"]);
        assert_eq!(words(r"'\'").unwrap(), ["\\"]);
        assert_eq!(words(r#"'x\"y'"#).unwrap(), [r#"x\"y"#]);
    }

    #[test]
    fn test_token_splitting_double_quoted() {
        assert_eq!(words(r#"foo "bar""#).unwrap(), ["foo", "bar"]);
        assert_eq!(words(r#""a"b"#).unwrap(), ["ab"]);
        assert_eq!(words(r#""a   b""#).unwrap(), ["a   b"]);
        assert_eq!(words(r#""""#).unwrap(), [""]);
        assert_eq!(words(r#""foo\"""#).unwrap(), ["foo\""]);
        assert_eq!(words(r#""foo\\""#).unwrap(), ["foo\\"]);
        assert_eq!(words(r#""a\b""#).unwrap(), ["a\\b"]);
        assert_eq!(words(r#""\$""#).unwrap(), ["$"]);
        assert_eq!(words(r#""\`""#).unwrap(), ["`"]);
        // `'` isn't special inside double quotes, so the backslash stays.
        assert_eq!(words(r#""x\'y""#).unwrap(), [r"x\'y"]);
    }

    #[test]
    fn test_token_splitting_mixed() {
        assert_eq!(
            words(r#"foo "bar"s b'c'"d""#).unwrap(),
            ["foo", "bars", "bcd"]
        );
        assert_eq!(words(r#"echo "it's""#).unwrap(), ["echo", "it's"]);
        assert_eq!(
            words(r#"echo 'say "hi"'"#).unwrap(),
            ["echo", r#"say "hi""#]
        );
        // The same `\\` in all three contexts.
        assert_eq!(words(r#"a\\b"c\\d"'e\\f'"#).unwrap(), [r"a\bc\de\\f"]);
    }

    #[test]
    fn test_token_splitting_operators() {
        // (input, expected tokens).
        let cases: &[(&str, Vec<Token>)] = &[
            (
                "echo foo > bar",
                vec![word("echo"), word("foo"), op(">"), word("bar")],
            ),
            (
                "echo foo 1> bar",
                vec![word("echo"), word("foo"), op("1>"), word("bar")],
            ),
            ("ls >> bar", vec![word("ls"), op(">>"), word("bar")]),
            ("ls 1>> bar", vec![word("ls"), op("1>>"), word("bar")]),
            (
                "type fizzbuzz 2> bar",
                vec![word("type"), word("fizzbuzz"), op("2>"), word("bar")],
            ),
            (
                "type fizzbuzz 2>> bar",
                vec![word("type"), word("fizzbuzz"), op("2>>"), word("bar")],
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(&split_tokens(input).unwrap(), expected, "input: {input:?}");
        }
    }

    #[test]
    fn test_token_splitting_malformed() {
        assert!(words("foo'").is_err());
        assert!(words("\"foo").is_err());
        // Note: REPL will never see this because read_line keeps the `\n`
        assert!(words("foo\\").is_err());
    }

    fn word(s: &str) -> Token {
        Token::Word(s.to_string())
    }

    fn op(s: &str) -> Token {
        match s {
            ">" | "1>" => Token::Operator(Operator::RedirectStdout),
            "2>" => Token::Operator(Operator::RedirectStderr),
            ">>" | "1>>" => Token::Operator(Operator::AppendStdout),
            "2>>" => Token::Operator(Operator::AppendStderr),
            _ => panic!("invalid operator"),
        }
    }
}
