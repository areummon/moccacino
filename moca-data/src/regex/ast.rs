#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegexAst {
    Empty,
    Epsilon,
    Char(char),
    Concat(Box<RegexAst>, Box<RegexAst>),
    Union(Box<RegexAst>, Box<RegexAst>),
    Star(Box<RegexAst>),
    Plus(Box<RegexAst>),
    Quest(Box<RegexAst>),
}

impl RegexAst {
    pub fn concat(left: RegexAst, right: RegexAst) -> RegexAst {
        match (left, right) {
            (RegexAst::Empty, _) | (_, RegexAst::Empty) => RegexAst::Empty,
            (RegexAst::Epsilon, other) | (other, RegexAst::Epsilon) => other,
            (left, right) => RegexAst::Concat(Box::new(left), Box::new(right)),
        }
    }

    pub fn union(left: RegexAst, right: RegexAst) -> RegexAst {
        match (left, right) {
            (RegexAst::Empty, other) | (other, RegexAst::Empty) => other,
            (left, right) if left == right => left,
            (RegexAst::Epsilon, star @ RegexAst::Star(_)) | (star @ RegexAst::Star(_), RegexAst::Epsilon) => star,
            (left, right) => RegexAst::Union(Box::new(left), Box::new(right)),
        }
    }

    pub fn star(inner: RegexAst) -> RegexAst {
        match inner {
            RegexAst::Empty | RegexAst::Epsilon => RegexAst::Epsilon,
            star @ RegexAst::Star(_) => star,
            inner => RegexAst::Star(Box::new(inner)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Union,
    Concat,
    Postfix,
    Atom,
}

fn precedence(node: &RegexAst) -> Precedence {
    match node {
        RegexAst::Union(_, _) => Precedence::Union,
        RegexAst::Concat(_, _) => Precedence::Concat,
        RegexAst::Star(_) | RegexAst::Plus(_) | RegexAst::Quest(_) => Precedence::Postfix,
        RegexAst::Empty | RegexAst::Epsilon | RegexAst::Char(_) => Precedence::Atom,
    }
}

fn is_operator_char(c: char) -> bool {
    matches!(c, '(' | ')' | '|' | '*' | '+' | '?' | '\\' | 'ε')
}

impl std::fmt::Display for RegexAst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", render(self, precedence(self)))
    }
}

fn render(node: &RegexAst, context: Precedence) -> String {
    let parens = |inner: &RegexAst| -> String {
        format!("({})", render(inner, Precedence::Union))
    };
    match node {
        RegexAst::Empty => "\u{2205}".to_string(),
        RegexAst::Epsilon => "ε".to_string(),
        RegexAst::Char(c) => {
            if is_operator_char(*c) {
                format!("\\{c}")
            } else {
                c.to_string()
            }
        },
        RegexAst::Concat(left, right) => {
            let body = format!(
                "{}{}",
                render(left, Precedence::Concat),
                render(right, Precedence::Concat)
            );
            if precedence(node) < context {
                format!("({body})")
            } else {
                body
            }
        },
        RegexAst::Union(left, right) => {
            let body = format!(
                "{}|{}",
                render(left, Precedence::Union),
                render(right, Precedence::Union)
            );
            if precedence(node) < context {
                format!("({body})")
            } else {
                body
            }
        },
        RegexAst::Star(inner) | RegexAst::Plus(inner) | RegexAst::Quest(inner) => {
            let operator = match node {
                RegexAst::Star(_) => "*",
                RegexAst::Plus(_) => "+",
                _ => "?",
            };
            let body = if precedence(inner) < Precedence::Atom {
                parens(inner)
            } else {
                render(inner, Precedence::Postfix)
            };
            format!("{body}{operator}")
        },
    }
}
