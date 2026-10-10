//! web/i18n.rs — bahasa Indonesia / English.
//!
//! Dua lapis, sengaja terpisah:
//!   * bahasa UNDANGAN (`invitations.lang`, migrasi 040) — dipilih pasangan
//!     untuk tamunya; tamu boleh mengganti sendiri lewat `?lang=` (tombol ID/EN
//!     di undangan);
//!   * bahasa SITUS (beranda, katalog, /buat …) — tahap berikutnya.
//!
//! Teks ditulis BERPASANGAN di tempat dipakai lewat [`tx!`] — tanpa berkas
//! kunci terpisah, tanpa biaya runtime, dan teks yang lupa diterjemahkan
//! langsung gagal kompilasi (makro wajib dua argumen).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lang {
    #[default]
    Id,
    En,
}

impl Lang {
    /// "id" / "en" (kode lain → None).
    pub fn from_code(s: &str) -> Option<Lang> {
        match s.trim().to_ascii_lowercase().as_str() {
            "id" => Some(Lang::Id),
            "en" => Some(Lang::En),
            _ => None,
        }
    }

    /// Kolom `invitations.lang` (kosong/rusak → Indonesia).
    pub fn of(code: &str) -> Lang {
        Lang::from_code(code).unwrap_or_default()
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::Id => "id",
            Lang::En => "en",
        }
    }

    pub fn other(self) -> Lang {
        match self {
            Lang::Id => Lang::En,
            Lang::En => Lang::Id,
        }
    }
}

/// Bahasa aktif dari context Leptos (disediakan InvShell untuk undangan);
/// di luar undangan (pratinjau admin, katalog) = Indonesia.
pub fn lang() -> Lang {
    leptos::prelude::use_context::<Lang>().unwrap_or_default()
}

/// Teks sesuai bahasa: `tx!(lang, "Buka Undangan", "Open Invitation")`.
/// Bekerja untuk `&'static str` maupun `String` (kedua cabang sama tipenya).
#[macro_export]
macro_rules! tx {
    ($lang:expr, $id:expr, $en:expr $(,)?) => {
        match $lang {
            $crate::web::i18n::Lang::Id => $id,
            $crate::web::i18n::Lang::En => $en,
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kode_bahasa() {
        assert_eq!(Lang::of("EN"), Lang::En);
        assert_eq!(Lang::of(""), Lang::Id);
        assert_eq!(Lang::of("fr"), Lang::Id);
        assert_eq!(Lang::En.other(), Lang::Id);
        assert_eq!(tx!(Lang::En, "Buka", "Open"), "Open");
    }
}
