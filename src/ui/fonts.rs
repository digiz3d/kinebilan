use sdl3::ttf::{Font, Sdl3TtfContext, sys};

const EMOJI_FONTS: &[&str] = &[
    r"C:\Windows\Fonts\seguiemj.ttf",
    "/System/Library/Fonts/Apple Color Emoji.ttc",
    "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
];

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
    pub body_bold: Font<'static>,
    pub body_large: Font<'static>,
    pub small: Font<'static>,
    pub badge: Font<'static>,
    pub button: Font<'static>,
    pub button_large: Font<'static>,
    _emoji: Vec<Font<'static>>,
}

impl Fonts {
    /// Fonts are rasterized at `scale` times their logical size so they stay sharp on HiDPI displays.
    pub fn load(ttf: &Sdl3TtfContext, scale: f32) -> Result<Self, String> {
        let mut emoji = Vec::new();
        Ok(Self {
            title: load_styled(ttf, BOLD_FONTS, 26.0 * scale, &mut emoji)?,
            heading: load_styled(ttf, BOLD_FONTS, 20.0 * scale, &mut emoji)?,
            section: load_styled(ttf, BOLD_FONTS, 16.0 * scale, &mut emoji)?,
            modal_title: load_styled(ttf, BOLD_FONTS, 17.0 * scale, &mut emoji)?,
            label: load_styled(ttf, BOLD_FONTS, 13.0 * scale, &mut emoji)?,
            body: load_styled(ttf, REGULAR_FONTS, 14.0 * scale, &mut emoji)?,
            body_bold: load_styled(ttf, BOLD_FONTS, 14.0 * scale, &mut emoji)?,
            body_large: load_styled(ttf, REGULAR_FONTS, 15.0 * scale, &mut emoji)?,
            small: load_styled(ttf, REGULAR_FONTS, 13.0 * scale, &mut emoji)?,
            badge: load_styled(ttf, REGULAR_FONTS, 12.0 * scale, &mut emoji)?,
            button: load_styled(ttf, REGULAR_FONTS, 14.0 * scale, &mut emoji)?,
            button_large: load_styled(ttf, BOLD_FONTS, 15.0 * scale, &mut emoji)?,
            _emoji: emoji,
        })
    }
}

fn load_styled(
    ttf: &Sdl3TtfContext,
    paths: &[&str],
    size: f32,
    emoji: &mut Vec<Font<'static>>,
) -> Result<Font<'static>, String> {
    let font = load_font(ttf, paths, size)?;
    let emoji_font = load_font(ttf, EMOJI_FONTS, size)?;
    unsafe { sys::TTF_AddFallbackFont(font.raw(), emoji_font.raw()) };
    emoji.push(emoji_font);
    Ok(font)
}

fn load_font(ttf: &Sdl3TtfContext, paths: &[&str], size: f32) -> Result<Font<'static>, String> {
    for path in paths {
        if let Ok(font) = ttf.load_font(path, size) {
            return Ok(font);
        }
    }
    Err(format!("aucune police système trouvée parmi : {}", paths.join(", ")))
}
