#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_normalized(red: f64, green: f64, blue: f64) -> Self {
        Self {
            r: normalized_channel(red),
            g: normalized_channel(green),
            b: normalized_channel(blue),
        }
    }

    pub fn from_hex_input(value: &str) -> Option<Self> {
        let hex = value.trim().trim_start_matches('#');

        if hex.len() != 6 || !hex.chars().all(|char| char.is_ascii_hexdigit()) {
            return None;
        }

        Some(Self {
            r: u8::from_str_radix(&hex[0..2], 16).ok()?,
            g: u8::from_str_radix(&hex[2..4], 16).ok()?,
            b: u8::from_str_radix(&hex[4..6], 16).ok()?,
        })
    }

    pub fn brightness(self) -> f64 {
        (0.299 * self.r as f64 + 0.587 * self.g as f64 + 0.114 * self.b as f64) / 255.0
    }

    pub fn hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn rgb(self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    pub fn rgba(self) -> String {
        format!("rgba({}, {}, {}, 1)", self.r, self.g, self.b)
    }

    pub fn hsl(self) -> String {
        let (hue, saturation, lightness) = self.hsl_components();
        format!("hsl({}, {}%, {}%)", hue, saturation, lightness)
    }

    pub fn css_variable(self) -> String {
        format!("--color: {};", self.hex())
    }

    pub fn format(self, format: CopyFormat) -> String {
        match format {
            CopyFormat::Hex => self.hex(),
            CopyFormat::Rgb => self.rgb(),
            CopyFormat::Rgba => self.rgba(),
            CopyFormat::Hsl => self.hsl(),
            CopyFormat::CssVariable => self.css_variable(),
        }
    }

    fn hsl_components(self) -> (i32, i32, i32) {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        let lightness = (max + min) / 2.0;

        if delta == 0.0 {
            return (0, 0, (lightness * 100.0).round() as i32);
        }

        let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
        let hue = if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        (
            hue.rem_euclid(360.0).round() as i32,
            (saturation * 100.0).round() as i32,
            (lightness * 100.0).round() as i32,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CopyFormat {
    Hex,
    Rgb,
    Rgba,
    Hsl,
    CssVariable,
}

impl CopyFormat {
    pub const ALL: [Self; 5] = [
        Self::Hex,
        Self::Rgb,
        Self::Rgba,
        Self::Hsl,
        Self::CssVariable,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Hex => "HEX",
            Self::Rgb => "RGB",
            Self::Rgba => "RGBA",
            Self::Hsl => "HSL",
            Self::CssVariable => "CSS variable",
        }
    }
}

fn normalized_channel(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub fn parse_rgb_channel(value: &str) -> Option<u8> {
    value.trim().parse::<u8>().ok()
}
