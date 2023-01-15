use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

const NAMED_CSS_COLOR_MAX_LENGTH: usize = 20;

static NAMED_CSS_COLORS: Lazy<HashMap<&'static [u8], RGBA>> = Lazy::new(|| {
    macro_rules! rgb {
        ($red: expr, $green: expr, $blue: expr) => {
            RGBA {
                red: $red,
                green: $green,
                blue: $blue,
                alpha: 255,
            }
        };
    }

    let mut m: HashMap<&'static [u8], _> = HashMap::new();

    m.insert(b"aliceblue", rgb!(240, 248, 255));
    m.insert(b"antiquewhite", rgb!(250, 235, 215));
    m.insert(b"aqua", rgb!(0, 255, 255));
    m.insert(b"aquamarine", rgb!(127, 255, 212));
    m.insert(b"azure", rgb!(240, 255, 255));
    m.insert(b"beige", rgb!(245, 245, 220));
    m.insert(b"bisque", rgb!(255, 228, 196));
    m.insert(b"black", rgb!(0, 0, 0));
    m.insert(b"blanchedalmond", rgb!(255, 235, 205));
    m.insert(b"blue", rgb!(0, 0, 255));
    m.insert(b"blueviolet", rgb!(138, 43, 226));
    m.insert(b"brown", rgb!(165, 42, 42));
    m.insert(b"burlywood", rgb!(222, 184, 135));
    m.insert(b"cadetblue", rgb!(95, 158, 160));
    m.insert(b"chartreuse", rgb!(127, 255, 0));
    m.insert(b"chocolate", rgb!(210, 105, 30));
    m.insert(b"coral", rgb!(255, 127, 80));
    m.insert(b"cornflowerblue", rgb!(100, 149, 237));
    m.insert(b"cornsilk", rgb!(255, 248, 220));
    m.insert(b"crimson", rgb!(220, 20, 60));
    m.insert(b"cyan", rgb!(0, 255, 255));
    m.insert(b"darkblue", rgb!(0, 0, 139));
    m.insert(b"darkcyan", rgb!(0, 139, 139));
    m.insert(b"darkgoldenrod", rgb!(184, 134, 11));
    m.insert(b"darkgray", rgb!(169, 169, 169));
    m.insert(b"darkgreen", rgb!(0, 100, 0));
    m.insert(b"darkgrey", rgb!(169, 169, 169));
    m.insert(b"darkkhaki", rgb!(189, 183, 107));
    m.insert(b"darkmagenta", rgb!(139, 0, 139));
    m.insert(b"darkolivegreen", rgb!(85, 107, 47));
    m.insert(b"darkorange", rgb!(255, 140, 0));
    m.insert(b"darkorchid", rgb!(153, 50, 204));
    m.insert(b"darkred", rgb!(139, 0, 0));
    m.insert(b"darksalmon", rgb!(233, 150, 122));
    m.insert(b"darkseagreen", rgb!(143, 188, 143));
    m.insert(b"darkslateblue", rgb!(72, 61, 139));
    m.insert(b"darkslategray", rgb!(47, 79, 79));
    m.insert(b"darkslategrey", rgb!(47, 79, 79));
    m.insert(b"darkturquoise", rgb!(0, 206, 209));
    m.insert(b"darkviolet", rgb!(148, 0, 211));
    m.insert(b"deeppink", rgb!(255, 20, 147));
    m.insert(b"deepskyblue", rgb!(0, 191, 255));
    m.insert(b"dimgray", rgb!(105, 105, 105));
    m.insert(b"dimgrey", rgb!(105, 105, 105));
    m.insert(b"dodgerblue", rgb!(30, 144, 255));
    m.insert(b"firebrick", rgb!(178, 34, 34));
    m.insert(b"floralwhite", rgb!(255, 250, 240));
    m.insert(b"forestgreen", rgb!(34, 139, 34));
    m.insert(b"fuchsia", rgb!(255, 0, 255));
    m.insert(b"gainsboro", rgb!(220, 220, 220));
    m.insert(b"ghostwhite", rgb!(248, 248, 255));
    m.insert(b"gold", rgb!(255, 215, 0));
    m.insert(b"goldenrod", rgb!(218, 165, 32));
    m.insert(b"gray", rgb!(128, 128, 128));
    m.insert(b"green", rgb!(0, 128, 0));
    m.insert(b"greenyellow", rgb!(173, 255, 47));
    m.insert(b"grey", rgb!(128, 128, 128));
    m.insert(b"honeydew", rgb!(240, 255, 240));
    m.insert(b"hotpink", rgb!(255, 105, 180));
    m.insert(b"indianred", rgb!(205, 92, 92));
    m.insert(b"indigo", rgb!(75, 0, 130));
    m.insert(b"ivory", rgb!(255, 255, 240));
    m.insert(b"khaki", rgb!(240, 230, 140));
    m.insert(b"lavender", rgb!(230, 230, 250));
    m.insert(b"lavenderblush", rgb!(255, 240, 245));
    m.insert(b"lawngreen", rgb!(124, 252, 0));
    m.insert(b"lemonchiffon", rgb!(255, 250, 205));
    m.insert(b"lightblue", rgb!(173, 216, 230));
    m.insert(b"lightcoral", rgb!(240, 128, 128));
    m.insert(b"lightcyan", rgb!(224, 255, 255));
    m.insert(b"lightgoldenrodyellow", rgb!(250, 250, 210));
    m.insert(b"lightgray", rgb!(211, 211, 211));
    m.insert(b"lightgreen", rgb!(144, 238, 144));
    m.insert(b"lightgrey", rgb!(211, 211, 211));
    m.insert(b"lightpink", rgb!(255, 182, 193));
    m.insert(b"lightsalmon", rgb!(255, 160, 122));
    m.insert(b"lightseagreen", rgb!(32, 178, 170));
    m.insert(b"lightskyblue", rgb!(135, 206, 250));
    m.insert(b"lightslategray", rgb!(119, 136, 153));
    m.insert(b"lightslategrey", rgb!(119, 136, 153));
    m.insert(b"lightsteelblue", rgb!(176, 196, 222));
    m.insert(b"lightyellow", rgb!(255, 255, 224));
    m.insert(b"lime", rgb!(0, 255, 0));
    m.insert(b"limegreen", rgb!(50, 205, 50));
    m.insert(b"linen", rgb!(250, 240, 230));
    m.insert(b"magenta", rgb!(255, 0, 255));
    m.insert(b"maroon", rgb!(128, 0, 0));
    m.insert(b"mediumaquamarine", rgb!(102, 205, 170));
    m.insert(b"mediumblue", rgb!(0, 0, 205));
    m.insert(b"mediumorchid", rgb!(186, 85, 211));
    m.insert(b"mediumpurple", rgb!(147, 112, 219));
    m.insert(b"mediumseagreen", rgb!(60, 179, 113));
    m.insert(b"mediumslateblue", rgb!(123, 104, 238));
    m.insert(b"mediumspringgreen", rgb!(0, 250, 154));
    m.insert(b"mediumturquoise", rgb!(72, 209, 204));
    m.insert(b"mediumvioletred", rgb!(199, 21, 133));
    m.insert(b"midnightblue", rgb!(25, 25, 112));
    m.insert(b"mintcream", rgb!(245, 255, 250));
    m.insert(b"mistyrose", rgb!(255, 228, 225));
    m.insert(b"moccasin", rgb!(255, 228, 181));
    m.insert(b"navajowhite", rgb!(255, 222, 173));
    m.insert(b"navy", rgb!(0, 0, 128));
    m.insert(b"oldlace", rgb!(253, 245, 230));
    m.insert(b"olive", rgb!(128, 128, 0));
    m.insert(b"olivedrab", rgb!(107, 142, 35));
    m.insert(b"orange", rgb!(255, 165, 0));
    m.insert(b"orangered", rgb!(255, 69, 0));
    m.insert(b"orchid", rgb!(218, 112, 214));
    m.insert(b"palegoldenrod", rgb!(238, 232, 170));
    m.insert(b"palegreen", rgb!(152, 251, 152));
    m.insert(b"paleturquoise", rgb!(175, 238, 238));
    m.insert(b"palevioletred", rgb!(219, 112, 147));
    m.insert(b"papayawhip", rgb!(255, 239, 213));
    m.insert(b"peachpuff", rgb!(255, 218, 185));
    m.insert(b"peru", rgb!(205, 133, 63));
    m.insert(b"pink", rgb!(255, 192, 203));
    m.insert(b"plum", rgb!(221, 160, 221));
    m.insert(b"powderblue", rgb!(176, 224, 230));
    m.insert(b"purple", rgb!(128, 0, 128));
    m.insert(b"rebeccapurple", rgb!(102, 51, 153));
    m.insert(b"red", rgb!(255, 0, 0));
    m.insert(b"rosybrown", rgb!(188, 143, 143));
    m.insert(b"royalblue", rgb!(65, 105, 225));
    m.insert(b"saddlebrown", rgb!(139, 69, 19));
    m.insert(b"salmon", rgb!(250, 128, 114));
    m.insert(b"sandybrown", rgb!(244, 164, 96));
    m.insert(b"seagreen", rgb!(46, 139, 87));
    m.insert(b"seashell", rgb!(255, 245, 238));
    m.insert(b"sienna", rgb!(160, 82, 45));
    m.insert(b"silver", rgb!(192, 192, 192));
    m.insert(b"skyblue", rgb!(135, 206, 235));
    m.insert(b"slateblue", rgb!(106, 90, 205));
    m.insert(b"slategray", rgb!(112, 128, 144));
    m.insert(b"slategrey", rgb!(112, 128, 144));
    m.insert(b"snow", rgb!(255, 250, 250));
    m.insert(b"springgreen", rgb!(0, 255, 127));
    m.insert(b"steelblue", rgb!(70, 130, 180));
    m.insert(b"tan", rgb!(210, 180, 140));
    m.insert(b"teal", rgb!(0, 128, 128));
    m.insert(b"thistle", rgb!(216, 191, 216));
    m.insert(b"tomato", rgb!(255, 99, 71));
    m.insert(b"turquoise", rgb!(64, 224, 208));
    m.insert(b"violet", rgb!(238, 130, 238));
    m.insert(b"wheat", rgb!(245, 222, 179));
    m.insert(b"white", rgb!(255, 255, 255));
    m.insert(b"whitesmoke", rgb!(245, 245, 245));
    m.insert(b"yellow", rgb!(255, 255, 0));
    m.insert(b"yellowgreen", rgb!(154, 205, 50));

    m.insert(
        b"transparent",
        RGBA {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 255,
        },
    );

    m
});

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct RGBA {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl fmt::Display for RGBA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}{}",
            self.red,
            self.green,
            self.blue,
            if self.alpha != 255 {
                format!("{:02x}", self.alpha)
            } else {
                "".to_owned()
            }
        )
    }
}

impl From<RGBA> for [f32; 4] {
    fn from(color: RGBA) -> [f32; 4] {
        [
            (color.red as f32 / 255_f32),
            (color.green as f32 / 255_f32),
            (color.blue as f32 / 255_f32),
            (color.alpha as f32 / 255_f32),
        ]
    }
}

impl Into<image::Rgba<u8>> for RGBA {
    fn into(self) -> image::Rgba<u8> {
        image::Rgba([self.red, self.green, self.blue, self.alpha])
    }
}

impl Into<image::Rgb<u8>> for RGBA {
    fn into(self) -> image::Rgb<u8> {
        image::Rgb([self.red, self.green, self.blue])
    }
}

#[derive(Debug, PartialEq)]
pub enum ColorParseError {
    InvalidInput,
}

impl FromStr for RGBA {
    type Err = ColorParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let red;
        let green;
        let blue;
        // default to no transparency
        let mut alpha = 255u8;

        let mut chars = string.chars();

        if chars.next() == Some('#') {
            // ignore # at the front
            let color_part_length = string.len() - 1;

            match color_part_length {
                3 => {
                    red = u8::from_str_radix(&chars.next().unwrap().to_string(), 16)?;
                    green = u8::from_str_radix(&chars.next().unwrap().to_string(), 16)?;
                    blue = u8::from_str_radix(&chars.next().unwrap().to_string(), 16)?;
                }
                6 | 8 => {
                    red = u8::from_str_radix(
                        &format!("{}{}", chars.next().unwrap(), chars.next().unwrap()),
                        16,
                    )?;
                    green = u8::from_str_radix(
                        &format!("{}{}", chars.next().unwrap(), chars.next().unwrap()),
                        16,
                    )?;
                    blue = u8::from_str_radix(
                        &format!("{}{}", chars.next().unwrap(), chars.next().unwrap()),
                        16,
                    )?;

                    if color_part_length == 8 {
                        alpha = u8::from_str_radix(
                            &format!("{}{}", chars.next().unwrap(), chars.next().unwrap()),
                            16,
                        )?;
                    }
                }
                _ => return Err(ColorParseError::InvalidInput),
            }
        } else if string.len() <= NAMED_CSS_COLOR_MAX_LENGTH {
            if let Some(&color) = NAMED_CSS_COLORS.get(string.to_ascii_lowercase().as_bytes()) {
                return Ok(color);
            } else {
                return Err(ColorParseError::InvalidInput);
            }
        } else {
            return Err(ColorParseError::InvalidInput);
        }

        Ok(RGBA {
            red,
            green,
            blue,
            alpha,
        })
    }
}

impl From<ParseIntError> for ColorParseError {
    fn from(_: ParseIntError) -> Self {
        ColorParseError::InvalidInput
    }
}

#[cfg(test)]
mod tests {

    use super::RGBA;

    #[test]
    pub fn test_parse() {
        let black = "#000";
        let bg_color = "#f1eee8";

        assert_eq!(
            Ok(RGBA {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255
            }),
            black.parse::<RGBA>()
        );
        assert_eq!(
            Ok(RGBA {
                red: 241,
                green: 238,
                blue: 232,
                alpha: 255
            }),
            bg_color.parse::<RGBA>()
        );
    }

    #[test]
    pub fn test_to_string() {
        let color = RGBA {
            red: 241,
            green: 238,
            blue: 232,
            alpha: 255,
        };

        assert_eq!("#f1eee8", color.to_string());
        assert_eq!(color.to_string().parse::<RGBA>(), Ok(color));
    }

    #[test]
    pub fn test_to_string_with_transparency() {
        let color = RGBA {
            red: 225,
            green: 236,
            blue: 244,
            alpha: 178,
        };

        assert_eq!("#e1ecf4b2", color.to_string());

        assert_eq!(color.to_string().parse::<RGBA>(), Ok(color));
    }

    #[test]
    pub fn test_to_float_array() {
        let bg_color = "yellow";

        let rgba = bg_color.parse::<RGBA>().unwrap();

        assert_eq!(
            rgba,
            RGBA {
                red: 255,
                green: 255,
                blue: 0,
                alpha: 255,
            }
        );

        let float: [f32; 4] = rgba.into();
        assert_eq!([1.0, 1.0, 0.0, 1.0], float);
    }
}
