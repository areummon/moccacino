/* Abstract syntax tree of the regular expressions supported by the library.
 *
 * Supported syntax (documented for users of the parser):
 * - characters match themselves; `ε` matches the empty string
 * - `\c` escapes any character c literally (e.g. `\*`, `\(`, `\\`)
 * - concatenation is juxtaposition: `ab`
 * - alternation: `a|b`
 * - repetition: `a*` (zero or more), `a+` (one or more), `a?` (optional)
 * - grouping with parentheses: `(ab)*`
 * - precedence, loosest to tightest: `|`, then concatenation, then * + ?
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegexAst {
    /* Matches nothing at all (the empty language). Not producible by the
     * parser, only constructible programmatically. */
    Empty,
    /* Matches the empty string. */
    Epsilon,
    Char(char),
    Concat(Box<RegexAst>, Box<RegexAst>),
    Union(Box<RegexAst>, Box<RegexAst>),
    Star(Box<RegexAst>),
    Plus(Box<RegexAst>),
    Quest(Box<RegexAst>),
}

/* Rendering precedence, loosest first. A child whose precedence is lower
 * than its parent's gets parenthesized. */
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

/* Characters that must be escaped to render as literals. */
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
        RegexAst::Empty => "\u{2205}".to_string(), // ∅; not reparseable
        RegexAst::Epsilon => "ε".to_string(),
        RegexAst::Char(c) => {
            if is_operator_char(*c) {
                format!("\\{}", c)
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
                format!("({})", body)
            } else {
                body
            }
        },
        RegexAst::Union(left, right) => {
            // Associative, so children never need parens: a flat chain
            // re-associates to the same language.
            let body = format!(
                "{}|{}",
                render(left, Precedence::Union),
                render(right, Precedence::Union)
            );
            if precedence(node) < context {
                format!("({})", body)
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
            // The operand of a postfix operator must read as one atom.
            let body = if precedence(inner) < Precedence::Atom {
                parens(inner)
            } else {
                render(inner, Precedence::Postfix)
            };
            format!("{}{}", body, operator)
        },
    }
}
