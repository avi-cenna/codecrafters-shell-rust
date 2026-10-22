use anyhow::{Result, bail};

pub fn split_tokens(s: &str) -> Result<Vec<String>> {
    let mut l = Lexer::new(s);
    l.split_tokens()
}

struct Lexer {
    text: String,
    pos: usize,
}

// This has to handle single quoting. Double quoting is in later stage...
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
                '\'' => result.push_str(&self.single_quoted()?),
                '"' => result.push_str(&self.double_quoted()?),
                _ if c.is_ascii_whitespace() => break,
                _ => result.push_str(&self.unquoted()?),
            }
        }
        Ok(result)
    }

    // TODO: n2 is grammar correct now?
    // unquoted ::= ([^'"\s] | '\' .)+
    // e38: The code now accepts `\` followed by any character, even a space or a quote, which
    // this class excludes. How would you write "a plain character, or a backslash followed by
    // any one character" as an alternation inside the `+`?
    fn unquoted(&mut self) -> Result<String> {
        let mut fragment = String::new();
        while let Some(c) = self.peek()
            && !c.is_ascii_whitespace()
            && !matches!(c, '\'' | '"')
        {
            if c == '\\' {
                self.bump();
                let Some(next) = self.peek() else {
                    bail!("Malformed input: expecting character after backslash");
                };
                fragment.push(next);
            } else {
                fragment.push(c);
            }
            self.bump();
        }
        Ok(fragment)
    }

    // single_quoted ::= "'" [^']* "'"
    fn single_quoted(&mut self) -> Result<String> {
        assert_eq!(self.peek(), Some('\''));
        self.bump();
        let mut fragment = String::new();
        while let Some(c) = self.peek()
            && c != '\''
        {
            fragment.push(c);
            self.bump();
        }
        let closing_quote = self.bump();
        if !matches!(closing_quote, Some('\'')) {
            bail!("Malformed input")
        }
        Ok(fragment)
    }

    // double_quoted ::= "\"" [^"]* "\""
    // Note: eventually will have to also support escape characters in this
    fn double_quoted(&mut self) -> Result<String> {
        assert_eq!(self.peek(), Some('"'));
        self.bump();
        let mut fragment = String::new();
        while let Some(c) = self.peek()
            && c != '"'
        {
            fragment.push(c);
            self.bump();
        }
        let closing_quote = self.bump();
        if !matches!(closing_quote, Some('"')) {
            bail!("Malformed input")
        }
        Ok(fragment)
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
    }

    #[test]
    fn test_token_splitting_single_quoted() {
        assert_eq!(split_tokens("foo 'bar'").unwrap(), ["foo", "bar"]);
        assert_eq!(split_tokens("'a'b").unwrap(), ["ab"]);
        assert_eq!(split_tokens("'a   b'").unwrap(), ["a   b"]);
        assert_eq!(split_tokens("''").unwrap(), [""]);
    }

    #[test]
    fn test_token_splitting_double_quoted() {
        assert_eq!(split_tokens(r#"foo "bar""#).unwrap(), ["foo", "bar"]);
        assert_eq!(split_tokens(r#""a"b"#).unwrap(), ["ab"]);
        assert_eq!(split_tokens(r#""a   b""#).unwrap(), ["a   b"]);
        assert_eq!(split_tokens(r#""""#).unwrap(), [""]);
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
    }

    #[test]
    fn test_token_splitting_malformed() {
        assert!(split_tokens("foo'").is_err());
        assert!(split_tokens("\"foo").is_err());
    }
}
