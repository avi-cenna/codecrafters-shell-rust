use anyhow::{Result, bail};

pub fn split_tokens(s: &str) -> Result<Vec<String>> {
    let mut l = Lexer::new(s);
    l.split_tokens()
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
            // MAYBE: consider save Vec<char>. It may make some of hte ops easier
        }
    }

    // input ::= skip (token skip)*
    fn split_tokens(&mut self) -> Result<Vec<String>> {
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

    // token ::= (single_quoted | double_quoted | unquoted)+
    fn token(&mut self) -> Result<String> {
        let mut result = String::new();
        while let Some(c) = self.peek() {
            match c {
                '\'' => self.single_quoted(&mut result)?,
                '"' => self.double_quoted(&mut result)?,
                _ if c.is_ascii_whitespace() => break,
                _ => self.unquoted(&mut result)?,
            }
        }
        Ok(result)
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

    #[test]
    fn test_token_splitting_unquoted() {
        assert_eq!(split_tokens("foo").unwrap(), ["foo"]);
        assert_eq!(split_tokens("echo hi").unwrap(), ["echo", "hi"]);
        assert_eq!(split_tokens(" john\tdoe  ").unwrap(), ["john", "doe"]);
        assert_eq!(split_tokens(r"foo\ ").unwrap(), ["foo "]);
        assert_eq!(split_tokens(r"foo\ bar").unwrap(), ["foo bar"]);
        assert_eq!(split_tokens(r"foo\   bar").unwrap(), ["foo ", "bar"]);
        assert_eq!(split_tokens(r#"foo\""#).unwrap(), ["foo\""]);
        // Outside quotes a backslash always disappears, whatever follows it.
        assert_eq!(split_tokens(r"a\b").unwrap(), ["ab"]);
        assert_eq!(split_tokens(r"\\").unwrap(), ["\\"]);
        assert_eq!(split_tokens(r"\'").unwrap(), ["'"]);
    }

    #[test]
    fn test_token_splitting_single_quoted() {
        assert_eq!(split_tokens("foo 'bar'").unwrap(), ["foo", "bar"]);
        assert_eq!(split_tokens("'a'b").unwrap(), ["ab"]);
        assert_eq!(split_tokens("'a   b'").unwrap(), ["a   b"]);
        assert_eq!(split_tokens("''").unwrap(), [""]);
        // Backslash is literal inside single quotes, even before the closing quote.
        assert_eq!(split_tokens(r"'a\b'").unwrap(), [r"a\b"]);
        assert_eq!(split_tokens(r"'\'").unwrap(), ["\\"]);
        assert_eq!(split_tokens(r#"'x\"y'"#).unwrap(), [r#"x\"y"#]);
    }

    #[test]
    fn test_token_splitting_double_quoted() {
        assert_eq!(split_tokens(r#"foo "bar""#).unwrap(), ["foo", "bar"]);
        assert_eq!(split_tokens(r#""a"b"#).unwrap(), ["ab"]);
        assert_eq!(split_tokens(r#""a   b""#).unwrap(), ["a   b"]);
        assert_eq!(split_tokens(r#""""#).unwrap(), [""]);
        assert_eq!(split_tokens(r#""foo\"""#).unwrap(), ["foo\""]);
        assert_eq!(split_tokens(r#""foo\\""#).unwrap(), ["foo\\"]);
        assert_eq!(split_tokens(r#""a\b""#).unwrap(), ["a\\b"]);
        assert_eq!(split_tokens(r#""\$""#).unwrap(), ["$"]);
        assert_eq!(split_tokens(r#""\`""#).unwrap(), ["`"]);
        // `'` isn't special inside double quotes, so the backslash stays.
        assert_eq!(split_tokens(r#""x\'y""#).unwrap(), [r"x\'y"]);
    }

    #[test]
    fn test_token_splitting_mixed() {
        assert_eq!(
            split_tokens(r#"foo "bar"s b'c'"d""#).unwrap(),
            ["foo", "bars", "bcd"]
        );
        assert_eq!(split_tokens(r#"echo "it's""#).unwrap(), ["echo", "it's"]);
        assert_eq!(
            split_tokens(r#"echo 'say "hi"'"#).unwrap(),
            ["echo", r#"say "hi""#]
        );
        // The same `\\` in all three contexts.
        assert_eq!(
            split_tokens(r#"a\\b"c\\d"'e\\f'"#).unwrap(),
            [r"a\bc\de\\f"]
        );
    }

    #[test]
    fn test_token_splitting_malformed() {
        assert!(split_tokens("foo'").is_err());
        assert!(split_tokens("\"foo").is_err());
        // Note: REPL will never see this because read_line keeps the `\n`
        assert!(split_tokens("foo\\").is_err());
    }
}
