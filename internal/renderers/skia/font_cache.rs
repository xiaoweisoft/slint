// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

use i_slint_common::sharedfontique::HashedBlob;
use i_slint_core::textlayout::sharedparley::parley;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct FontCache {
    font_mgr: skia_safe::FontMgr,
    fonts: HashMap<(HashedBlob, u32), Option<skia_safe::Typeface>>,
}

impl Default for FontCache {
    fn default() -> Self {
        Self { font_mgr: skia_safe::FontMgr::new(), fonts: Default::default() }
    }
}

impl FontCache {
    pub fn font(&mut self, font: &parley::FontData) -> Option<skia_safe::Typeface> {
        self.fonts
            .entry((font.data.clone().into(), font.index))
            .or_insert_with(|| {
                let bytes = font.data.as_ref();
                // Font blobs own stable immutable storage. SkData retains that
                // owner independently of this cache and any typeface clones.
                let data = unsafe {
                    skia_safe::Data::from_owned_bytes(
                        font.data.clone(),
                        bytes.as_ptr(),
                        bytes.len(),
                    )
                };
                let typeface = self.font_mgr.new_from_owned_data(
                    data,
                    if font.index > 0 { Some(font.index as _) } else { None },
                );

                // Due to  https://issues.skia.org/issues/310510989, fonts from true type collections
                // with an index > 0 fail to load on macOS. As a workaround, we manually extract the font from the
                // collection and load it as a single font.
                #[cfg(target_vendor = "apple")]
                if font.index > 0 && typeface.is_none() {
                    {
                        if let Some(typeface) = read_fonts::CollectionRef::new(font.data.as_ref())
                            .ok()
                            .and_then(|ttc| ttc.get(font.index).ok())
                            .map(|ttf| {
                                write_fonts::FontBuilder::new().copy_missing_tables(ttf).build()
                            })
                            .and_then(|new_ttf| self.font_mgr.new_from_data(&new_ttf, None))
                        {
                            return Some(typeface);
                        }
                    }
                }

                typeface
            })
            .clone()
    }
}

thread_local! {
    pub static FONT_CACHE: RefCell<FontCache> = RefCell::new(Default::default())
}

#[cfg(test)]
mod ownership_tests {
    #[test]
    fn owned_typeface_preserves_collection_indices_and_glyphs() {
        let Ok(path) = std::env::var("SLINT_TEST_TTC_FONT") else { return };
        let bytes = std::sync::Arc::new(std::fs::read(path).unwrap());
        let mgr = skia_safe::FontMgr::new();
        for index in 0..3 {
            let original = mgr.new_from_data(&bytes, Some(index)).unwrap();
            let data = unsafe {
                skia_safe::Data::from_owned_bytes(bytes.clone(), bytes.as_ptr(), bytes.len())
            };
            let shared = mgr.new_from_owned_data(data, Some(index)).unwrap();
            assert_eq!(original.family_name(), shared.family_name());
            assert_eq!(original.count_glyphs(), shared.count_glyphs());
            for ch in "English中文繁體日本語한국어".chars() {
                assert_eq!(
                    original.unichar_to_glyph(ch as i32),
                    shared.unichar_to_glyph(ch as i32)
                );
            }
        }
    }
}
