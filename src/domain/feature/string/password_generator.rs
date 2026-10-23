use ring::rand::{SecureRandom, SystemRandom};

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};
use crate::domain::shared::format::{number, rows};

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "password_generator",
    section: "string",
    title: "Password Generator",
    description: "Generate strong passwords and estimate their entropy",
};

pub const MAX_LENGTH: usize = 256;
pub const MAX_COUNT: usize = 100;

const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
/// Characters easily confused with one another.
const AMBIGUOUS: &str = "Il1|O0o`'\"";

/// Offline attack against a fast hash, in guesses per second.
const GUESSES_PER_SECOND: f64 = 1e10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PasswordOptions {
    pub length: usize,
    pub lowercase: bool,
    pub uppercase: bool,
    pub digits: bool,
    pub symbols: bool,
    pub exclude_ambiguous: bool,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 20,
            lowercase: true,
            uppercase: true,
            digits: true,
            symbols: true,
            exclude_ambiguous: false,
        }
    }
}

impl PasswordOptions {
    /// The selected character classes, ambiguous characters removed if asked.
    fn classes(&self) -> Vec<Vec<char>> {
        [
            (self.lowercase, LOWERCASE),
            (self.uppercase, UPPERCASE),
            (self.digits, DIGITS),
            (self.symbols, SYMBOLS),
        ]
        .into_iter()
        .filter(|(on, _)| *on)
        .map(|(_, chars)| {
            chars
                .chars()
                .filter(|c| !(self.exclude_ambiguous && AMBIGUOUS.contains(*c)))
                .collect()
        })
        .collect()
    }

    pub fn pool_size(&self) -> usize {
        self.classes().iter().map(Vec::len).sum()
    }

    /// Entropy of a password drawn with these options, in bits.
    pub fn entropy_bits(&self) -> f64 {
        self.length as f64 * (self.pool_size().max(1) as f64).log2()
    }
}

/// Uniform index below `n`, from the system's secure random generator
/// (rejection sampling, no modulo bias).
pub fn secure_below(n: usize) -> usize {
    let rng = SystemRandom::new();
    let n = n as u64;
    let zone = u64::MAX - (u64::MAX % n);
    loop {
        let mut bytes = [0u8; 8];
        rng.fill(&mut bytes).expect("system random generator");
        let value = u64::from_le_bytes(bytes);
        if value < zone {
            return (value % n) as usize;
        }
    }
}

/// `count` passwords, one per line, each holding at least one character of
/// every selected class. `below(n)` must return a uniform index below `n`.
pub fn generate_with(
    options: PasswordOptions,
    count: usize,
    below: &mut impl FnMut(usize) -> usize,
) -> DomainResult<String> {
    let classes = options.classes();
    if classes.is_empty() {
        return Err(DomainError::new("Select at least one kind of character."));
    }
    if options.length < classes.len() || options.length > MAX_LENGTH {
        return Err(DomainError::new(format!(
            "Length must be between {} and {MAX_LENGTH}.",
            classes.len()
        )));
    }
    let pool: Vec<char> = classes.concat();
    let passwords: Vec<String> = (0..count.clamp(1, MAX_COUNT))
        .map(|_| {
            loop {
                let password: Vec<char> = (0..options.length)
                    .map(|_| pool[below(pool.len())])
                    .collect();
                // Redraw rather than patch, to keep every valid password equally likely.
                if classes
                    .iter()
                    .all(|class| password.iter().any(|c| class.contains(c)))
                {
                    break password.into_iter().collect();
                }
            }
        })
        .collect();
    Ok(passwords.join("\n"))
}

pub fn generate(options: PasswordOptions, count: usize) -> DomainResult<String> {
    generate_with(options, count, &mut secure_below)
}

pub fn strength(bits: f64) -> &'static str {
    match bits {
        b if b < 28.0 => "Very weak",
        b if b < 36.0 => "Weak",
        b if b < 60.0 => "Reasonable",
        b if b < 128.0 => "Strong",
        _ => "Very strong",
    }
}

/// "3 hours", "12 thousand years"…
pub fn crack_time(bits: f64) -> String {
    // On average, half the space is searched.
    let seconds = 2f64.powf(bits - 1.0) / GUESSES_PER_SECOND;
    let units = [
        (31_557_600.0, "year"),
        (86_400.0, "day"),
        (3600.0, "hour"),
        (60.0, "minute"),
        (1.0, "second"),
    ];
    let Some((size, unit)) = units.iter().find(|(size, _)| seconds >= *size) else {
        return "instantly".into();
    };
    let value = seconds / size;
    let (value, scale) = [
        (1e12, " trillion"),
        (1e9, " billion"),
        (1e6, " million"),
        (1e3, " thousand"),
    ]
    .into_iter()
    .find(|(s, _)| value >= *s && *unit == "year")
    .map_or((value, ""), |(s, name)| (value / s, name));
    if value >= 1e3 {
        return "longer than the age of the universe".into();
    }
    let value = value.floor().max(1.0);
    let plural = if value == 1.0 && scale.is_empty() {
        ""
    } else {
        "s"
    };
    format!("{value}{scale} {unit}{plural}")
}

/// Very common passwords and keyboard walks, checked as substrings.
const COMMON: [&str; 16] = [
    "password", "123456", "qwerty", "azerty", "letmein", "welcome", "admin", "iloveyou", "monkey",
    "dragon", "abc123", "111111", "000000", "passw0rd", "secret", "sunshine",
];

/// Entropy estimate of an existing password, from the character classes it
/// uses, with warnings for obvious weaknesses.
pub fn inspect(password: &str) -> String {
    let chars: Vec<char> = password.chars().collect();
    let has = |set: &str| chars.iter().any(|c| set.contains(*c));
    let other = chars.iter().any(|c| !c.is_ascii() || c.is_ascii_control());
    let pool = [
        (has(LOWERCASE), 26),
        (has(UPPERCASE), 26),
        (has(DIGITS), 10),
        (has(SYMBOLS) || chars.contains(&' '), 33),
        (other, 100),
    ]
    .iter()
    .filter(|(used, _)| *used)
    .map(|(_, size)| size)
    .sum::<usize>();
    let mut bits = chars.len() as f64 * (pool.max(1) as f64).log2();

    let mut warnings = Vec::new();
    let lower = password.to_lowercase();
    if let Some(word) = COMMON.iter().find(|w| lower.contains(**w)) {
        warnings.push(format!("contains the common password \"{word}\""));
        bits = bits.min(10.0);
    }
    let unique = {
        let mut sorted = chars.clone();
        sorted.sort();
        sorted.dedup();
        sorted.len()
    };
    if chars.len() > 2 && unique <= chars.len() / 3 {
        warnings.push("many repeated characters".into());
        bits = bits.min(unique as f64 * 4.0);
    }
    if chars.len() < 12 {
        warnings.push("shorter than 12 characters".into());
    }

    let mut lines = vec![
        ("Length".to_string(), format!("{} characters", chars.len())),
        ("Charset".into(), format!("{pool} possible characters")),
        (
            "Entropy".into(),
            format!("≈ {} bits, {}", number(bits, 1), strength(bits)),
        ),
        (
            "Crack time".into(),
            format!("{} at 10 billion guesses/s", crack_time(bits)),
        ),
    ];
    for warning in warnings {
        lines.push(("Warning".into(), warning));
    }
    rows(&lines)
}
