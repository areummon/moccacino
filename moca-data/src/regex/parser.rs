use super::ast::RegexAst;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub position: usize,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (at position {})", self.message, self.position)
    }
}

impl std::error::Error for ParseError {}

pub fn parse(pattern: &str) -> Result<RegexAst, ParseError> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut parser = Parser { chars: &chars, pos: 0 };
    let ast = parser.union()?;
    if parser.peek() == Some(')') {
        return Err(parser.error("unmatched ')'"));
    }
    Ok(ast)
}

struct Parser<'a> {
    chars: &'a [char],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn eat(&mut self) {
        self.pos += 1;
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            position: self.pos,
            message: message.to_string(),
        }
    }

    fn union(&mut self) -> Result<RegexAst, ParseError> {
        let mut left = self.concat()?;
        while self.peek() == Some('|') {
            self.eat();
            let right = self.concat()?;
            left = RegexAst::Union(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn concat(&mut self) -> Result<RegexAst, ParseError> {
        let mut nodes: Vec<RegexAst> = Vec::new();
        loop {
            match self.peek() {
                None | Some('|') | Some(')') => break,
                _ => nodes.push(self.repeat()?),
            }
        }
        Ok(nodes.into_iter().rev().reduce(|right, left| {
            RegexAst::Concat(Box::new(left), Box::new(right))
        }).unwrap_or(RegexAst::Epsilon))
    }

    fn repeat(&mut self) -> Result<RegexAst, ParseError> {
        let mut node = self.atom()?;
        loop {
            match self.peek() {
                Some('*') => {
                    self.eat();
                    node = RegexAst::Star(Box::new(node));
                },
                Some('+') => {
                    self.eat();
                    node = RegexAst::Plus(Box::new(node));
                },
                Some('?') => {
                    self.eat();
                    node = RegexAst::Quest(Box::new(node));
                },
                _ => return Ok(node),
            }
        }
    }

    fn atom(&mut self) -> Result<RegexAst, ParseError> {
        match self.peek() {
            Some('(') => {
                self.eat();
                let inner = self.union()?;
                if self.peek() != Some(')') {
                    return Err(self.error("expected ')' to close the group"));
                }
                self.eat();
                Ok(inner)
            },
            Some('\\') => {
                self.eat();
                match self.bump() {
                    Some(c) => Ok(RegexAst::Char(c)),
                    None => Err(self.error("escape at the end of the pattern")),
                }
            },
            Some('ε') => {
                self.eat();
                Ok(RegexAst::Epsilon)
            },
            Some(c @ ('*' | '+' | '?')) => {
                let message = format!(
                    "repetition operator '{}' without an expression to repeat",
                    c
                );
                Err(self.error(&message))
            },
            Some(_) => {
                let c = self.bump().unwrap();
                Ok(RegexAst::Char(c))
            },
            None => Err(self.error("expected an expression")),
        }
    }
}
