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
    fonts.sort_by(|a, b| (a.label.to_lowercase(), &a.name).cmp(&(b.label.to_lowercase(), &b.name)));
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

#[cfg(target_os = "macos")]
mod imp {
    use super::FontFamily;
    use core_foundation::base::{CFType, TCFType};
    use core_foundation::string::CFString;
    use core_text::font_collection;
    use core_text::font_descriptor::{
        kCTFontFamilyNameAttribute, CTFontDescriptor, CTFontDescriptorCopyLocalizedAttribute,
        SymbolicTraitAccessors, TraitAccessors,
    };

    pub fn system_fonts() -> Result<Vec<FontFamily>, ()> {
        let collection = font_collection::create_for_all_families();
        let Some(descriptors) = collection.get_descriptors() else {
            return Ok(Vec::new());
        };
        let mut fonts = Vec::new();
        for descriptor in descriptors.iter() {
            // The family name is what CSS `font-family` matches; names starting with "." are
            // the system's own interface fonts, which pages cannot ask for by name.
            let name = descriptor.family_name();
            if name.starts_with('.') {
                continue;
            }
            let label = localized_family(&descriptor).unwrap_or_else(|| name.clone());
            let monospace = descriptor.traits().symbolic_traits().is_monospace();
            fonts.push(FontFamily { name, label, monospace });
        }
        Ok(fonts)
    }

    /// The family name in the user's language (ヒラギノ角ゴシック for Hiragino Sans), when it has one.
    fn localized_family(descriptor: &CTFontDescriptor) -> Option<String> {
        unsafe {
            let value = CTFontDescriptorCopyLocalizedAttribute(
                descriptor.as_concrete_TypeRef(),
                kCTFontFamilyNameAttribute,
                std::ptr::null_mut(),
            );
            if value.is_null() {
                return None;
            }
            CFType::wrap_under_create_rule(value).downcast::<CFString>().map(|s| s.to_string())
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod mac_tests {
    #[test]
    fn lists_installed_fonts() {
        let fonts = super::system_fonts();
        println!("{} families", fonts.len());
        for f in fonts.iter().filter(|f| f.label != f.name).take(8) {
            println!("{} / {}", f.label, f.name);
        }
        let mono: Vec<_> = fonts.iter().filter(|f| f.monospace).map(|f| f.label.as_str()).collect();
        println!("monospace ({}): {}", mono.len(), mono.join(", "));
        assert!(fonts.iter().any(|f| f.name == "Menlo" && f.monospace));
        assert!(fonts.iter().any(|f| f.name == "Helvetica" && !f.monospace));
        assert!(fonts.iter().all(|f| !f.name.starts_with('.')));
        let names: Vec<_> = fonts.iter().map(|f| f.name.as_str()).collect();
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(names.len(), unique.len(), "each family is listed once");
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod imp {
    pub fn system_fonts() -> Result<Vec<super::FontFamily>, ()> {
        Ok(Vec::new())
    }
}
