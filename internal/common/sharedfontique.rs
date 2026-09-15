// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

pub use fontique;
pub use ttf_parser;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub static COLLECTION: std::sync::LazyLock<Collection> = std::sync::LazyLock::new(|| {
    let mut collection = fontique::Collection::new(fontique::CollectionOptions {
        shared: true,
        ..Default::default()
    });

    let mut source_cache = fontique::SourceCache::new_shared();

    let mut default_fonts: HashMap<std::path::PathBuf, fontique::QueryFont> = Default::default();

    #[cfg(any(target_family = "wasm", target_os = "nto"))]
    {
        let data = include_bytes!("sharedfontique/DejaVuSans.ttf");
        let fonts = collection.register_fonts(fontique::Blob::new(Arc::new(data)), None);
        for script in fontique::Script::all_samples().iter().map(|(script, _)| *script) {
            collection.append_fallbacks(
                fontique::FallbackKey::new(script, None),
                fonts.iter().map(|(family_id, _)| *family_id),
            );
        }
        for generic_family in [
            fontique::GenericFamily::SansSerif,
            fontique::GenericFamily::SystemUi,
            fontique::GenericFamily::UiSansSerif,
        ] {
            collection.append_generic_families(
                generic_family,
                fonts.iter().map(|(family_id, _)| *family_id),
            );
        }
    }

    let mut add_font_from_path = |path: std::path::PathBuf| {
        if let Ok(blob) = font_blob_from_path(&path) {
            let fonts = collection.register_fonts(blob, None);
            for generic_family in [
                fontique::GenericFamily::SansSerif,
                fontique::GenericFamily::SystemUi,
                fontique::GenericFamily::UiSansSerif,
            ] {
                collection.set_generic_families(
                    generic_family,
                    fonts.iter().map(|(family_id, _)| *family_id),
                );
            }

            // just use the first font of the first family in the file.
            if let Some(font) = fonts.first().and_then(|(id, infos)| {
                let info = infos.first()?;
                get_font_for_info(&mut collection, &mut source_cache, *id, &info)
            }) {
                default_fonts.insert(path, font);
            }
        }
    };

    if let Some(path) = std::env::var_os("SLINT_DEFAULT_FONT") {
        let path = std::path::Path::new(&path);
        if path.extension().is_some() {
            add_font_from_path(path.to_owned());
        } else if let Ok(dir) = std::fs::read_dir(path) {
            for file in dir {
                if let Ok(file) = file {
                    add_font_from_path(file.path());
                }
            }
        }
    }

    Collection { inner: collection, source_cache, default_fonts: Arc::new(default_fonts) }
});

pub fn get_collection() -> Collection {
    COLLECTION.clone()
}

/// Register a font file once per canonical path and keep its bytes file-backed.
///
/// `std::fs::read()` turns each registered font into anonymous heap memory in
/// every Slint process. On small-memory Linux images the same Mio font files are
/// used by HOME, overlay, text input, and lockscreen, so path registration must
/// use the kernel's file-backed page cache instead.
pub fn register_font_from_path(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    static REGISTERED: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<PathBuf>>> =
        std::sync::LazyLock::new(Default::default);

    let canonical_path = canonical_font_path(path);
    let mut registered = REGISTERED.lock().unwrap_or_else(|poison| poison.into_inner());
    if registered.insert(canonical_path.clone()) {
        let blob = font_blob_from_path(&canonical_path)?;
        get_collection().register_fonts(blob, None);
    }
    Ok(())
}

fn font_blob_from_path(path: &Path) -> Result<fontique::Blob<u8>, Box<dyn std::error::Error>> {
    static MMAPPED_FONTS: std::sync::LazyLock<
        std::sync::Mutex<HashMap<PathBuf, Arc<memmap2::Mmap>>>,
    > = std::sync::LazyLock::new(Default::default);

    let canonical_path = canonical_font_path(path);
    let mut mapped_fonts = MMAPPED_FONTS.lock().unwrap_or_else(|poison| poison.into_inner());
    if let Some(mapping) = mapped_fonts.get(&canonical_path) {
        return Ok(fontique::Blob::new(mapping.clone()));
    }

    let file = std::fs::File::open(&canonical_path)?;
    // SAFETY: The mapping is read-only and Slint treats registered font bytes as
    // immutable. The Arc stored in MMAPPED_FONTS keeps the mapping alive for the
    // lifetime of every fontique blob handed to renderer caches.
    let mapping = Arc::new(unsafe { memmap2::MmapOptions::new().map(&file)? });
    mapped_fonts.insert(canonical_path, mapping.clone());
    Ok(fontique::Blob::new(mapping))
}

fn canonical_font_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.into())
}

/// Register immutable embedded fonts once across temporary renderer lifetimes.
/// The static byte slice stays in the executable's file-backed mapping instead
/// of being copied to an anonymous Vec for every window.
pub fn register_font_from_memory(data: &'static [u8]) {
    static REGISTERED: std::sync::LazyLock<
        std::sync::Mutex<std::collections::HashSet<(usize, usize)>>,
    > = std::sync::LazyLock::new(Default::default);
    let mut registered = REGISTERED.lock().unwrap_or_else(|poison| poison.into_inner());
    if registered.insert((data.as_ptr() as usize, data.len())) {
        get_collection().register_fonts(fontique::Blob::new(Arc::new(data)), None);
    }
}

#[cfg(test)]
mod registration_tests {
    use super::*;

    #[test]
    fn repeated_registration_preserves_font_identity_and_static_storage() {
        static FONT: &[u8] = include_bytes!("sharedfontique/DejaVuSans.ttf");
        register_font_from_memory(FONT);
        let lookup = || {
            let mut collection = get_collection();
            let mut query = collection.query();
            query.set_families([fontique::QueryFamily::Named("DejaVu Sans")]);
            let mut found = None;
            query.matches_with(|font| {
                found = Some(font.clone());
                fontique::QueryStatus::Stop
            });
            found.expect("embedded family must remain available")
        };
        let first = lookup();
        assert_eq!(first.blob.data().as_ptr(), FONT.as_ptr());
        for _ in 0..100 {
            register_font_from_memory(FONT);
            let next = lookup();
            assert_eq!(next.blob.id(), first.blob.id());
            assert_eq!(next.index, first.index);
        }
    }

    #[test]
    fn repeated_path_blob_uses_same_file_mapping() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("sharedfontique/DejaVuSans.ttf");
        let first = font_blob_from_path(&path).expect("test font maps");
        let second = font_blob_from_path(&path).expect("test font maps again");
        assert_eq!(first.data().as_ptr(), second.data().as_ptr());
        assert_eq!(first.data().len(), second.data().len());
        assert_eq!(first.data()[..4], second.data()[..4]);
    }
}

#[derive(Clone)]
pub struct Collection {
    pub inner: fontique::Collection,
    pub source_cache: fontique::SourceCache,
    pub default_fonts: Arc<HashMap<PathBuf, fontique::QueryFont>>,
}

impl Collection {
    pub fn query<'a>(&'a mut self) -> fontique::Query<'a> {
        self.inner.query(&mut self.source_cache)
    }

    pub fn get_font_for_info(
        &mut self,
        family_id: fontique::FamilyId,
        info: &fontique::FontInfo,
    ) -> Option<fontique::QueryFont> {
        get_font_for_info(&mut self.inner, &mut self.source_cache, family_id, info)
    }
}

fn get_font_for_info(
    collection: &mut fontique::Collection,
    source_cache: &mut fontique::SourceCache,
    family_id: fontique::FamilyId,
    info: &fontique::FontInfo,
) -> Option<fontique::QueryFont> {
    let mut query = collection.query(source_cache);
    query.set_families(std::iter::once(fontique::QueryFamily::from(family_id)));
    query.set_attributes(fontique::Attributes {
        weight: info.weight(),
        style: info.style(),
        width: info.width(),
    });
    let mut font = None;
    query.matches_with(|queried_font| {
        font = Some(queried_font.clone());
        fontique::QueryStatus::Stop
    });
    font
}

impl std::ops::Deref for Collection {
    type Target = fontique::Collection;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Collection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Font metrics in design space. Scale with desired pixel size and divided by units_per_em
/// to obtain pixel metrics.
#[derive(Clone)]
pub struct DesignFontMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub x_height: f32,
    pub cap_height: f32,
    pub units_per_em: f32,
}

impl DesignFontMetrics {
    pub fn new(font: &fontique::QueryFont) -> Self {
        let face = ttf_parser::Face::parse(font.blob.data(), font.index).unwrap();
        Self::new_from_face(&face)
    }

    pub fn new_from_face(face: &ttf_parser::Face) -> Self {
        Self {
            ascent: face.ascender() as f32,
            descent: face.descender() as f32,
            x_height: face.x_height().unwrap_or_default() as f32,
            cap_height: face.capital_height().unwrap_or_default() as f32,
            units_per_em: face.units_per_em() as f32,
        }
    }
}

pub const FALLBACK_FAMILIES: [fontique::GenericFamily; 2] = [
    // FemtoVG renderer needs SansSerif first, as it has difficulties rendering from SystemUi on macOS
    fontique::GenericFamily::SansSerif,
    fontique::GenericFamily::SystemUi,
];

/// Wraper around fontique::Blob to permit use of the blob as a key in the cache in the different renderers,
/// to map the blob to the native type face representation (skia_safe::Typeface, femtovg::FontId, QRawFont, etc.).
/// The use as key also ensures the blob remains strongly referenced, so that it doesn't vanish from the
/// shared SourceCache (parley prunes it).
pub struct HashedBlob(fontique::Blob<u8>);
impl core::hash::Hash for HashedBlob {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.id().hash(state);
    }
}

impl PartialEq for HashedBlob {
    fn eq(&self, other: &Self) -> bool {
        self.0.id() == other.0.id()
    }
}

impl Eq for HashedBlob {}

impl From<fontique::Blob<u8>> for HashedBlob {
    fn from(value: fontique::Blob<u8>) -> Self {
        Self(value)
    }
}

impl AsRef<fontique::Blob<u8>> for HashedBlob {
    fn as_ref(&self) -> &fontique::Blob<u8> {
        &self.0
    }
}
