use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};
use crate::domain::shared::format::{number, rows};

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "color",
    section: "convert",
    title: "Color Converter",
    description: "Convert HEX, RGB, HSL and OKLCH colors and check WCAG contrast",
};

/// An sRGB color, channels in 0..=1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub alpha: f64,
    /// Set when the source (OKLCH, OKLab) was outside sRGB and got clipped.
    pub clipped: bool,
}

const FORMATS_HINT: &str = "Accepted: #3584e4, #38e, rgb(53 132 228), rgba(53, 132, 228, 0.5), \
     hsl(213 76% 55%), oklch(60% 0.15 256), oklab(0.6 -0.03 -0.15)";

fn invalid(input: &str) -> DomainError {
    DomainError::new(format!("Unrecognized color: {input}\n{FORMATS_HINT}"))
}

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

impl Color {
    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        let in_gamut = |c: f64| (-1e-4..=1.0 + 1e-4).contains(&c);
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            alpha: 1.0,
            clipped: !(in_gamut(r) && in_gamut(g) && in_gamut(b)),
        }
    }

    fn with_alpha(mut self, alpha: f64) -> Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    pub fn to_rgb8(self) -> [u8; 3] {
        [self.r, self.g, self.b].map(|c| (c * 255.0).round() as u8)
    }

    pub fn hex(self) -> String {
        let [r, g, b] = self.to_rgb8();
        if self.alpha < 1.0 {
            let a = (self.alpha * 255.0).round() as u8;
            format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
        } else {
            format!("#{r:02X}{g:02X}{b:02X}")
        }
    }

    /// (hue in degrees, saturation, lightness), all but hue in 0..=1.
    pub fn hsl(self) -> (f64, f64, f64) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let l = (max + min) / 2.0;
        let d = max - min;
        if d < 1e-12 {
            return (0.0, 0.0, l);
        }
        let s = d / (1.0 - (2.0 * l - 1.0).abs());
        let h = if max == self.r {
            ((self.g - self.b) / d).rem_euclid(6.0)
        } else if max == self.g {
            (self.b - self.r) / d + 2.0
        } else {
            (self.r - self.g) / d + 4.0
        };
        (h * 60.0, s, l)
    }

    pub fn from_hsl(h: f64, s: f64, l: f64) -> Self {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let h = h.rem_euclid(360.0) / 60.0;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());
        let (r, g, b) = match h as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let m = l - c / 2.0;
        Self::rgb(r + m, g + m, b + m)
    }

    /// OKLab (L, a, b), from Björn Ottosson's reference matrices.
    pub fn oklab(self) -> (f64, f64, f64) {
        let [r, g, b] = [self.r, self.g, self.b].map(srgb_to_linear);
        let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
        let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
        let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
        (
            0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
            1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
            0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
        )
    }

    pub fn from_oklab(lightness: f64, a: f64, b: f64) -> Self {
        let l = (lightness + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
        let m = (lightness - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
        let s = (lightness - 0.089_484_177_5 * a - 1.291_485_548_0 * b).powi(3);
        let [r, g, b] = [
            4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
            -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
            -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s,
        ]
        .map(linear_to_srgb);
        Self::rgb(r, g, b)
    }

    /// OKLCH (L, chroma, hue in degrees).
    pub fn oklch(self) -> (f64, f64, f64) {
        let (l, a, b) = self.oklab();
        let c = a.hypot(b);
        let h = if c < 1e-4 {
            0.0
        } else {
            b.atan2(a).to_degrees().rem_euclid(360.0)
        };
        (l, c, h)
    }

    pub fn from_oklch(l: f64, c: f64, h: f64) -> Self {
        let h = h.to_radians();
        Self::from_oklab(l, c * h.cos(), c * h.sin())
    }

    /// WCAG 2.x relative luminance (alpha ignored).
    pub fn luminance(self) -> f64 {
        let [r, g, b] = [self.r, self.g, self.b].map(srgb_to_linear);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
}

/// WCAG 2.x contrast ratio, from 1 to 21.
pub fn contrast_ratio(a: Color, b: Color) -> f64 {
    let (la, lb) = (a.luminance(), b.luminance());
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

fn parse_hex(digits: &str) -> Option<Color> {
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let expanded: String = match digits.len() {
        3 | 4 => digits.chars().flat_map(|c| [c, c]).collect(),
        6 | 8 => digits.to_string(),
        _ => return None,
    };
    let byte = |i: usize| {
        u8::from_str_radix(&expanded[i..i + 2], 16)
            .ok()
            .map(|v| v as f64 / 255.0)
    };
    let color = Color::rgb(byte(0)?, byte(2)?, byte(4)?);
    Some(match expanded.len() {
        8 => color.with_alpha(byte(6)?),
        _ => color,
    })
}

/// A number, or a percentage of `full` when suffixed with `%`.
fn value(token: &str, full: f64) -> Option<f64> {
    match token.strip_suffix('%') {
        Some(p) => p.parse::<f64>().ok().map(|v| v / 100.0 * full),
        None => token.parse().ok(),
    }
}

fn hue(token: &str) -> Option<f64> {
    token.strip_suffix("deg").unwrap_or(token).parse().ok()
}

fn alpha(token: Option<&&str>) -> Option<f64> {
    token.map_or(Some(1.0), |t| value(t, 1.0))
}

fn parse_function(name: &str, args: &str) -> Option<Color> {
    let tokens: Vec<&str> = args
        .split([' ', ',', '/'])
        .filter(|t| !t.is_empty())
        .collect();
    if !(3..=4).contains(&tokens.len()) {
        return None;
    }
    let color = match name {
        "rgb" | "rgba" => {
            let channel = |t: &str| value(t, 255.0).map(|v| v / 255.0);
            Color::rgb(
                channel(tokens[0])?,
                channel(tokens[1])?,
                channel(tokens[2])?,
            )
        }
        "hsl" | "hsla" => {
            let fraction = |t: &str| value(t.trim_end_matches('%'), 100.0).map(|v| v / 100.0);
            Color::from_hsl(hue(tokens[0])?, fraction(tokens[1])?, fraction(tokens[2])?)
        }
        "oklch" => Color::from_oklch(
            value(tokens[0], 1.0)?,
            value(tokens[1], 0.4)?,
            hue(tokens[2])?,
        ),
        "oklab" => Color::from_oklab(
            value(tokens[0], 1.0)?,
            value(tokens[1], 0.4)?,
            value(tokens[2], 0.4)?,
        ),
        _ => return None,
    };
    Some(color.with_alpha(alpha(tokens.get(3))?))
}

/// Parses a CSS-like color: hex (with or without `#`), `rgb()`, `hsl()`,
/// `oklch()` or `oklab()`.
pub fn parse(input: &str) -> DomainResult<Color> {
    let text = input.trim().to_ascii_lowercase();
    if text.is_empty() {
        return Err(DomainError::new("Enter a color."));
    }
    let color = if let Some((name, rest)) = text.split_once('(') {
        rest.strip_suffix(')')
            .and_then(|args| parse_function(name.trim(), args))
    } else {
        parse_hex(text.strip_prefix('#').unwrap_or(&text))
    };
    color.ok_or_else(|| invalid(input.trim()))
}

/// `value` rounded to `decimals`, so that float noise prints as 0.
fn round(value: f64, decimals: i32) -> String {
    let scale = 10f64.powi(decimals);
    number((value * scale).round() / scale, decimals as usize)
}

fn with_alpha_suffix(color: Color) -> String {
    if color.alpha < 1.0 {
        format!(" / {}", round(color.alpha, 3))
    } else {
        String::new()
    }
}

fn verdict(ratio: f64, threshold: f64) -> &'static str {
    if ratio >= threshold { "✓" } else { "✗" }
}

/// "4.54:1  AA ✓  AAA ✗  AA large ✓  AAA large ✓"
pub fn contrast_summary(ratio: f64) -> String {
    format!(
        "{}:1  AA {}  AAA {}  AA large {}  AAA large {}",
        round(ratio, 2),
        verdict(ratio, 4.5),
        verdict(ratio, 7.0),
        verdict(ratio, 3.0),
        verdict(ratio, 4.5),
    )
}

/// Every notation of `input`, then its contrast against white, black and
/// `background` if given.
pub fn describe(input: &str, background: Option<&str>) -> DomainResult<String> {
    let color = parse(input)?;
    let a = with_alpha_suffix(color);
    let [r, g, b] = color.to_rgb8();
    let (h, s, l) = color.hsl();
    let (ok_l, ok_c, ok_h) = color.oklch();
    let (_, ok_a, ok_b) = color.oklab();
    let mut lines = vec![
        ("HEX".to_string(), color.hex()),
        ("RGB".into(), format!("rgb({r} {g} {b}{a})")),
        (
            "HSL".into(),
            format!(
                "hsl({} {}% {}%{a})",
                round(h, 1),
                round(s * 100.0, 1),
                round(l * 100.0, 1)
            ),
        ),
        (
            "OKLCH".into(),
            format!(
                "oklch({}% {} {}{a})",
                round(ok_l * 100.0, 2),
                round(ok_c, 4),
                round(ok_h, 2)
            ),
        ),
        (
            "OKLab".into(),
            format!(
                "oklab({}% {} {}{a})",
                round(ok_l * 100.0, 2),
                round(ok_a, 4),
                round(ok_b, 4)
            ),
        ),
        ("Luminance".into(), round(color.luminance(), 4)),
    ];
    if color.clipped {
        lines.push(("Gamut".into(), "outside sRGB, clipped".into()));
    }

    let white = Color::rgb(1.0, 1.0, 1.0);
    let black = Color::rgb(0.0, 0.0, 0.0);
    let mut contrast = vec![
        (
            "vs white".to_string(),
            contrast_summary(contrast_ratio(color, white)),
        ),
        (
            "vs black".into(),
            contrast_summary(contrast_ratio(color, black)),
        ),
    ];
    if let Some(bg) = background.map(str::trim).filter(|b| !b.is_empty()) {
        let other = parse(bg)?;
        contrast.push((
            format!("vs {}", other.hex()),
            contrast_summary(contrast_ratio(color, other)),
        ));
    }
    let note = if color.alpha < 1.0 {
        " (transparency ignored)"
    } else {
        ""
    };
    Ok(format!(
        "{}\n\nContrast, WCAG 2.x{note}\n{}",
        rows(&lines),
        rows(&contrast)
    ))
}
