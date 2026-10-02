//! web/icons.rs — ikon Material Symbols (sesuai desain Stitch) + QR SVG.
//!
//! Font ikon dimuat dari Google Fonts dengan `icon_names=` → hanya glyph yang
//! dipakai yang diunduh (belasan KB, bukan 3 MB). AKIBATNYA: setiap nama ikon
//! baru WAJIB ditambahkan ke `ICONS` (urut abjad), kalau tidak ikon tampil
//! sebagai TEKS. Test `semua_ikon_terdaftar` menjaga ini.

use leptos::prelude::*;

pub const ICONS: &[&str] = &[
    "ac_unit",
    "account_balance",
    "account_balance_wallet",
    "add",
    "add_circle",
    "add_photo_alternate",
    "admin_panel_settings",
    "album",
    "animation",
    "architecture",
    "arrow_back",
    "arrow_downward",
    "arrow_forward",
    "arrow_upward",
    "auto_awesome",
    "auto_stories",
    "badge",
    "bolt",
    "brush",
    "calculate",
    "calendar_add_on",
    "calendar_month",
    "call",
    "cancel",
    "celebration",
    "chat",
    "check",
    "check_circle",
    "checklist",
    "checkroom",
    "chevron_left",
    "chevron_right",
    "clean_hands",
    "close",
    "cloud_upload",
    "code",
    "content_copy",
    "credit_card",
    "dashboard",
    "deck",
    "delete",
    "diamond",
    "directions_car",
    "door_open",
    "download",
    "drafts",
    "eco",
    "edit_note",
    "electric_bolt",
    "event_available",
    "event_busy",
    "expand_more",
    "fact_check",
    "favorite",
    "flare",
    "forest",
    "graphic_eq",
    "grid_view",
    "groups",
    "help",
    "hourglass_top",
    "how_to_reg",
    "info",
    "inventory_2",
    "key",
    "layers",
    "library_music",
    "light_mode",
    "link",
    "local_florist",
    "local_parking",
    "local_shipping",
    "location_on",
    "lock",
    "logout",
    "loyalty",
    "mail",
    "manage_accounts",
    "map",
    "mark_email_read",
    "mic",
    "military_tech",
    "music_note",
    "near_me",
    "notifications_active",
    "open_in_new",
    "palette",
    "pause",
    "person",
    "person_add",
    "photo_camera",
    "photo_library",
    "play_arrow",
    "power",
    "public",
    "qr_code_2",
    "qr_code_scanner",
    "redeem",
    "report",
    "restart_alt",
    "savings",
    "schedule",
    "search",
    "send",
    "share",
    "shield",
    "shopping_bag",
    "smartphone",
    "spa",
    "speaker",
    "speaker_group",
    "star",
    "support_agent",
    "swap_horiz",
    "swipe_up",
    "table_restaurant",
    "texture",
    "thermostat",
    "trending_up",
    "tune",
    "umbrella",
    "verified",
    "verified_user",
    "view_carousel",
    "visibility",
    "volume_off",
    "volume_up",
    "water_drop",
    "workspace_premium",
    "yard",
];

/// URL CSS Google Fonts untuk subset ikon di atas.
pub fn icon_font_href() -> String {
    format!(
        "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..24,400,0..1,0&icon_names={}&display=block",
        ICONS.join(",")
    )
}

#[component]
pub fn Icon(name: &'static str, #[prop(optional)] class: &'static str) -> impl IntoView {
    view! { <span class=format!("ms {class}") aria-hidden="true">{icon_or_help(name)}</span> }
}

/// Nama ikon yang ada di subset font, atau "help". Ikon di luar subset akan
/// tampil sebagai TEKS mentah — ini menutup kelas bug itu untuk nama dinamis
/// yang tak terjangkau test `semua_ikon_terdaftar`.
pub fn icon_or_help(name: &str) -> &'static str {
    match ICONS.iter().find(|i| **i == name) {
        Some(i) => i,
        None => {
            #[cfg(debug_assertions)]
            leptos::logging::warn!("ikon `{name}` belum terdaftar di ICONS — tampil sebagai `help`");
            "help"
        }
    }
}

/// Glyph ikon untuk nama dinamis (String dari data/konten).
#[component]
pub fn DynIcon(#[prop(into)] name: String) -> impl IntoView {
    view! { <span class="ms" aria-hidden="true">{icon_or_help(&name)}</span> }
}

/// QR sebagai SVG inline (tanpa JS, tanpa gambar eksternal).
pub fn qr_svg(text: &str) -> String {
    use qrcodegen::{QrCode, QrCodeEcc};
    let Ok(qr) = QrCode::encode_text(text, QrCodeEcc::Medium) else {
        return String::new();
    };
    // Standar QR: zona sepi 4 modul — penting untuk pemindaian di meja
    // penerima tamu (cahaya redup, layar HP gelap).
    let border = 4;
    let size = qr.size();
    let dim = size + border * 2;
    let mut path = String::new();
    for y in 0..size {
        for x in 0..size {
            if qr.get_module(x, y) {
                path.push_str(&format!("M{},{}h1v1h-1z", x + border, y + border));
            }
        }
    }
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {dim} {dim}" shape-rendering="crispEdges" role="img" aria-label="Kode QR"><rect width="100%" height="100%" fill="#fff"/><path d="{path}" fill="#161d17"/></svg>"##
    )
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::ICONS;

    /// Pindai src/ untuk `Icon name="…"` dan string ikon di tabel data, lalu
    /// pastikan semuanya ada di `ICONS` (dan `ICONS` urut).
    #[test]
    fn semua_ikon_terdaftar() {
        let mut sorted = ICONS.to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted, ICONS, "ICONS harus urut abjad");

        let mut missing = Vec::new();
        let mut stack = vec![std::path::PathBuf::from("src")];
        while let Some(dir) = stack.pop() {
            for e in std::fs::read_dir(&dir).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                let src = std::fs::read_to_string(&p).unwrap();
                if p.ends_with("icons.rs") {
                    continue;
                }
                for marker in ["Icon name=\"", "icon=\"", "icon: \""] {
                    for part in src.split(marker).skip(1) {
                        let name = part.split('"').next().unwrap();
                        if !ICONS.contains(&name) {
                            missing.push(format!("{} → {name}", p.display()));
                        }
                    }
                }
            }
        }
        assert!(missing.is_empty(), "ikon belum terdaftar di ICONS: {missing:?}");
    }
}
