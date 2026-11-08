use serde::Serialize;

#[derive(Serialize)]
pub struct FontFamily {
    /// English family name, used as the CSS `font-family` value.
    pub name: String,
    /// Japanese name when the font has one, for display in the picker.
    pub label: String,
    /// Every glyph has the same advance width, so the time never shifts sideways.
    pub monospace: bool,
}

/// Installed font families, sorted by display label.
pub fn system_fonts() -> Vec<FontFamily> {
    let mut fonts = imp::system_fonts().unwrap_or_default();
    // "@" families are vertical-writing variants of CJK fonts.
    fonts.retain(|f| !f.name.is_empty() && !f.name.starts_with('@'));
    fonts.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));
    fonts.dedup_by(|a, b| a.name == b.name);
    fonts
}

#[cfg(windows)]
mod imp {
    use super::FontFamily;
    use windows::core::{w, Interface, BOOL, PCWSTR};
    use windows::Win32::Graphics::DirectWrite::{
        DWriteCreateFactory, IDWriteFactory, IDWriteFont1, IDWriteFontCollection,
        IDWriteFontFamily, IDWriteLocalizedStrings, DWRITE_FACTORY_TYPE_SHARED,
    };

    pub fn system_fonts() -> windows::core::Result<Vec<FontFamily>> {
        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            let mut collection: Option<IDWriteFontCollection> = None;
            factory.GetSystemFontCollection(&mut collection, false)?;
            let Some(collection) = collection else {
                return Ok(Vec::new());
            };

            let mut fonts = Vec::new();
            for i in 0..collection.GetFontFamilyCount() {
                let family = collection.GetFontFamily(i)?;
                let names = family.GetFamilyNames()?;
                let Some(name) = localized(&names, w!("en-us")).or_else(|| string_at(&names, 0))
                else {
                    continue;
                };
                let label = localized(&names, w!("ja-jp")).unwrap_or_else(|| name.clone());
                let monospace = is_monospace(&family);
                fonts.push(FontFamily { name, label, monospace });
            }
            Ok(fonts)
        }
    }

    unsafe fn is_monospace(family: &IDWriteFontFamily) -> bool {
        unsafe { family.GetFont(0) }
            .ok()
            .and_then(|font| font.cast::<IDWriteFont1>().ok())
            .is_some_and(|font| unsafe { font.IsMonospacedFont() }.as_bool())
    }

    unsafe fn localized(names: &IDWriteLocalizedStrings, locale: PCWSTR) -> Option<String> {
        let mut index = 0;
        let mut exists = BOOL(0);
        unsafe { names.FindLocaleName(locale, &mut index, &mut exists) }.ok()?;
        if exists.as_bool() {
            unsafe { string_at(names, index) }
        } else {
            None
        }
    }

    unsafe fn string_at(names: &IDWriteLocalizedStrings, index: u32) -> Option<String> {
        let len = unsafe { names.GetStringLength(index) }.ok()? as usize;
        let mut buf = vec![0u16; len + 1];
        unsafe { names.GetString(index, &mut buf) }.ok()?;
        Some(String::from_utf16_lossy(&buf[..len]))
    }
}

#[cfg(all(test, windows))]
mod tests {
    #[test]
    fn lists_installed_fonts() {
        let fonts = super::system_fonts();
        println!("{} families", fonts.len());
        for f in fonts.iter().filter(|f| f.label != f.name).take(8) {
            println!("{} / {}", f.label, f.name);
        }
        let mono: Vec<_> = fonts.iter().filter(|f| f.monospace).map(|f| f.label.as_str()).collect();
        println!("monospace ({}): {}", mono.len(), mono.join(", "));
        assert!(fonts.iter().any(|f| f.name == "Consolas" && f.monospace));
        assert!(fonts.iter().any(|f| f.name == "Segoe UI" && !f.monospace));
        assert!(fonts.iter().any(|f| f.label == "メイリオ" && f.name == "Meiryo"));
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn system_fonts() -> Result<Vec<super::FontFamily>, ()> {
        Ok(Vec::new())
    }
}
