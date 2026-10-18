use sdl3::ttf::{Font, Sdl3TtfContext};

const REGULAR_FONTS: &[&str] = &[
    r"C:\Windows\Fonts\segoeui.ttf",
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    "/Library/Fonts/Arial.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
];

const BOLD_FONTS: &[&str] = &[
    r"C:\Windows\Fonts\segoeuib.ttf",
    "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
    "/Library/Fonts/Arial Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
];

pub struct Fonts {
    pub title: Font<'static>,
    pub heading: Font<'static>,
    pub section: Font<'static>,
    pub modal_title: Font<'static>,
    pub label: Font<'static>,
    pub body: Font<'static>,
    pub body_large: Font<'static>,
    pub small: Font<'static>,
    pub badge: Font<'static>,
    pub button: Font<'static>,
    pub button_large: Font<'static>,
}

impl Fonts {
    pub fn load(ttf: &Sdl3TtfContext) -> Result<Self, String> {
        Ok(Self {
            title: load_font(ttf, BOLD_FONTS, 26.0)?,
            heading: load_font(ttf, BOLD_FONTS, 20.0)?,
            section: load_font(ttf, BOLD_FONTS, 16.0)?,
            modal_title: load_font(ttf, BOLD_FONTS, 17.0)?,
            label: load_font(ttf, BOLD_FONTS, 13.0)?,
            body: load_font(ttf, REGULAR_FONTS, 14.0)?,
            body_large: load_font(ttf, REGULAR_FONTS, 15.0)?,
            small: load_font(ttf, REGULAR_FONTS, 13.0)?,
            badge: load_font(ttf, REGULAR_FONTS, 12.0)?,
            button: load_font(ttf, REGULAR_FONTS, 14.0)?,
            button_large: load_font(ttf, BOLD_FONTS, 15.0)?,
        })
    }
}

fn load_font(ttf: &Sdl3TtfContext, paths: &[&str], size: f32) -> Result<Font<'static>, String> {
    for path in paths {
        if let Ok(font) = ttf.load_font(path, size) {
            return Ok(font);
        }
    }
    Err(format!("aucune police système trouvée parmi : {}", paths.join(", ")))
}
