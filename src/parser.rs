use crate::lexer::Token;
use anyhow::{Result, bail};
use std::io::Write;

pub fn parse_tokens(s: Vec<Token>) -> Result<ShellCmd> {
    let mut l = Parser::new(s);
    l.wip()
}

pub struct ShellCmd {
    pub name: String,
    pub args: Vec<String>,
    pub output_type: OutputType,
}

pub enum OutputType {
    Stdout,
    File(String),
}

// impl OutputType {
//     pub fn writeable(&self) -> Result<Box<dyn Write>> {
//         let writeable: Box<dyn Write> = match &self {
//             OutputType::File(f) => Box::new(std::fs::File::create(f)?),
//             OutputType::Stdout => Box::new(std::io::stdout()),
//         };
//         Ok(writeable)
//     }
// }

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(s: Vec<Token>) -> Self {
        Self { tokens: s, pos: 0 }
    }

    // input ::= cmd_name (arg)* redirection?
    // TODO: n2 help me here, something wrong again,.... should i have smaller functions for arg and cmd name?
    fn wip(&mut self) -> Result<ShellCmd> {
        assert!(!self.tokens.is_empty());

        let name = self.word()?;
        let mut args = vec![];
        while let Some(Token::Word(_)) = self.peek() {
            args.push(self.word()?);
        }

        let output_type = if let Some(Token::Operator(_)) = self.peek() {
            let r = self.redirection()?;
            OutputType::File(r)
        } else {
            OutputType::Stdout
        };

        Ok(ShellCmd {
            name,
            args,
            output_type,
        })
    }

    // fn cmd_name(&mut self) -> Result<String> {
    //     let x = self.bump();
    //     match x {
    //         Some(Token::Word(s)) => Ok(s.clone()),
    //         Some(Token::Operator(_)) => bail!("todo"),
    //         None => unreachable!(),
    //     }
    // }

    fn word(&mut self) -> Result<String> {
        match self.bump() {
            Some(Token::Word(s)) => Ok(s.clone()),
            Some(Token::Operator(_)) => bail!("todo"),
            None => unreachable!(),
        }
    }

    fn redirection(&mut self) -> Result<String> {
        assert!(matches!(self.peek(), Some(Token::Operator(_))));
        self.bump();
        match self.bump() {
            Some(Token::Word(s)) => Ok(s.clone()),
            Some(Token::Operator(_)) => {
                bail!("Malformed input: cannot have two operators in a row")
            }
            None => bail!("Malformed input: must have file have redirection"),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    // fn peek2(&self) -> Option<Token> {
    //     let it  = self.tokens.into_iter();
    //     // it.peekable().next()
    //     None
    // }

    fn bump(&mut self) -> Option<&Token> {
        let c = self.tokens.get(self.pos)?;
        self.pos += 1;
        Some(c)
    }
}
