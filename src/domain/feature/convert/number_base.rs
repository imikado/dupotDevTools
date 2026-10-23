use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};
use crate::domain::shared::format::rows;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "number_base",
    section: "convert",
    title: "Number Base",
    description: "Convert between binary, octal, decimal and hex, with bitwise operations",
};

pub const EXAMPLES: [&str; 4] = ["0xFF & 0b1010", "1 << 12 | 0o17", "~0x0F", "-42"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token {
    Num(i128),
    Op(&'static str),
    Open,
    Close,
}

const OPERATORS: [&str; 12] = ["<<", ">>", "|", "^", "&", "+", "-", "*", "/", "%", "~", "!"];

fn parse_number(literal: &str) -> DomainResult<i128> {
    let digits = literal.replace('_', "");
    let lower = digits.to_ascii_lowercase();
    let (radix, body) = match lower.get(..2) {
        Some("0x") => (16, &lower[2..]),
        Some("0b") => (2, &lower[2..]),
        Some("0o") => (8, &lower[2..]),
        _ => (10, lower.as_str()),
    };
    if body.is_empty() {
        return Err(DomainError::new(format!("Missing digits in {literal}")));
    }
    i128::from_str_radix(body, radix)
        .map_err(|_| DomainError::new(format!("Invalid number: {literal}")))
}

fn tokenize(expr: &str) -> DomainResult<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut rest = expr.trim();
    while let Some(c) = rest.chars().next() {
        if c.is_whitespace() {
            rest = &rest[c.len_utf8()..];
        } else if c.is_ascii_alphanumeric() {
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            tokens.push(Token::Num(parse_number(&rest[..end])?));
            rest = &rest[end..];
        } else if c == '(' || c == ')' {
            tokens.push(if c == '(' { Token::Open } else { Token::Close });
            rest = &rest[1..];
        } else if let Some(op) = OPERATORS.iter().find(|op| rest.starts_with(**op)) {
            tokens.push(Token::Op(op));
            rest = &rest[op.len()..];
        } else {
            return Err(DomainError::new(format!("Unexpected character: {c}")));
        }
    }
    Ok(tokens)
}

/// Binding power of binary operators, C precedence (higher binds tighter).
fn precedence(op: &str) -> Option<u8> {
    Some(match op {
        "|" => 1,
        "^" => 2,
        "&" => 3,
        "<<" | ">>" => 4,
        "+" | "-" => 5,
        "*" | "/" | "%" => 6,
        _ => return None,
    })
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

fn overflow() -> DomainError {
    DomainError::new("Overflow (values are limited to 128-bit signed integers).")
}

impl Parser {
    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).copied();
        self.pos += 1;
        token
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).copied()
    }

    fn unary(&mut self) -> DomainResult<i128> {
        match self.next() {
            Some(Token::Num(n)) => Ok(n),
            Some(Token::Op("-")) => self.unary()?.checked_neg().ok_or_else(overflow),
            Some(Token::Op("+")) => self.unary(),
            Some(Token::Op("~" | "!")) => Ok(!self.unary()?),
            Some(Token::Open) => {
                let value = self.binary(0)?;
                match self.next() {
                    Some(Token::Close) => Ok(value),
                    _ => Err(DomainError::new("Missing closing parenthesis.")),
                }
            }
            Some(Token::Close) => Err(DomainError::new("Unexpected ')'.")),
            Some(Token::Op(op)) => Err(DomainError::new(format!("Unexpected operator: {op}"))),
            None => Err(DomainError::new("Incomplete expression.")),
        }
    }

    fn binary(&mut self, min_precedence: u8) -> DomainResult<i128> {
        let mut left = self.unary()?;
        while let Some(Token::Op(op)) = self.peek() {
            let Some(p) = precedence(op).filter(|p| *p > min_precedence) else {
                break;
            };
            self.pos += 1;
            let right = self.binary(p)?;
            left = apply(op, left, right)?;
        }
        Ok(left)
    }
}

fn apply(op: &str, a: i128, b: i128) -> DomainResult<i128> {
    let shift = || {
        u32::try_from(b)
            .ok()
            .filter(|s| *s < 128)
            .ok_or_else(|| DomainError::new("Shift amount must be between 0 and 127."))
    };
    match op {
        "|" => Ok(a | b),
        "^" => Ok(a ^ b),
        "&" => Ok(a & b),
        "<<" => a.checked_shl(shift()?).ok_or_else(overflow),
        ">>" => a.checked_shr(shift()?).ok_or_else(overflow),
        "+" => a.checked_add(b).ok_or_else(overflow),
        "-" => a.checked_sub(b).ok_or_else(overflow),
        "*" => a.checked_mul(b).ok_or_else(overflow),
        "/" | "%" if b == 0 => Err(DomainError::new("Division by zero.")),
        "/" => a.checked_div(b).ok_or_else(overflow),
        _ => a.checked_rem(b).ok_or_else(overflow),
    }
}

/// Evaluates an integer expression: literals in any base (`255`, `0xFF`,
/// `0o377`, `0b1111_1111`) combined with `| ^ & << >> + - * / % ~` and
/// parentheses, with C precedence.
pub fn evaluate(expr: &str) -> DomainResult<i128> {
    let tokens = tokenize(expr)?;
    if tokens.is_empty() {
        return Err(DomainError::new("Enter a number or an expression."));
    }
    let mut parser = Parser { tokens, pos: 0 };
    let value = parser.binary(0)?;
    match parser.peek() {
        None => Ok(value),
        Some(Token::Close) => Err(DomainError::new("Unexpected ')'.")),
        Some(_) => Err(DomainError::new("Missing operator between values.")),
    }
}

/// Smallest of 8, 16, 32, 64 and 128 bits holding `value` (two's complement
/// when negative).
pub fn bit_width(value: i128) -> u32 {
    [8, 16, 32, 64]
        .into_iter()
        .find(|&w| {
            if value < 0 {
                value >= -(1i128 << (w - 1))
            } else {
                value < (1i128 << w)
            }
        })
        .unwrap_or(128)
}

/// `digits` split in groups of `size` from the right, joined by `sep`.
fn group(digits: &str, size: usize, sep: char) -> String {
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(size) {
            out.push(sep);
        }
        out.push(c);
    }
    out
}

/// Every base representation of the expression's value, one per line.
pub fn describe(expr: &str) -> DomainResult<String> {
    let value = evaluate(expr)?;
    let width = bit_width(value);
    // Two's complement for negatives, zero-extended to `width` otherwise.
    let bits = if width == 128 {
        value as u128
    } else {
        (value as u128) & ((1u128 << width) - 1)
    };
    let suffix = if value < 0 {
        format!(" (two's complement, {width}-bit)")
    } else {
        String::new()
    };
    let binary = format!("{bits:0width$b}", width = width as usize);
    let mut lines: Vec<(String, String)> = vec![
        ("Decimal".into(), value.to_string()),
        (format!("Hex{suffix}"), format!("0x{bits:X}")),
        (format!("Octal{suffix}"), format!("0o{bits:o}")),
        (format!("Binary{suffix}"), group(&binary, 4, ' ')),
    ];
    if value < 0 {
        lines.push(("Unsigned".into(), bits.to_string()));
    } else if width < 128 && bits >> (width - 1) == 1 {
        let signed = value - (1i128 << width);
        lines.push((format!("Signed ({width}-bit)"), signed.to_string()));
    }
    lines.push((
        "Bits".into(),
        format!(
            "{} set, {} significant, fits in {width} bits",
            bits.count_ones(),
            128 - bits.leading_zeros()
        ),
    ));
    if let Some(c) = u8::try_from(value)
        .ok()
        .filter(|b| b.is_ascii_graphic() || *b == b' ')
    {
        lines.push(("ASCII".into(), format!("'{}'", c as char)));
    }
    Ok(rows(&lines))
}
