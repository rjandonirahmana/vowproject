use std::{env, fs, path::Path};

fn main() {
    // ── Embed daftar migrasi ───────────────────────────────────────────────────
    // `migration/*.sql` di-embed ke binari sebagai (nama, isi), URUT NAMA FILE,
    // supaya container yang hanya memuat binari tetap bisa bermigrasi. Lihat
    // `src/config/migrate.rs`.
    let mig_dir = Path::new("migration");
    let mut migs: Vec<_> = fs::read_dir(mig_dir)
        .expect("read migration/")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "sql").unwrap_or(false))
        .collect();
    migs.sort();

    let mut list = String::from(
        "/// (nama berkas, isi SQL) — urut nama, di-generate build.rs.\n\
         pub static MIGRATIONS: &[(&str, &str)] = &[\n",
    );
    for p in &migs {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        let abs = fs::canonicalize(p).expect("canonicalize migration");
        list.push_str(&format!(
            "    ({:?}, include_str!({:?})),\n",
            name,
            abs.to_string_lossy()
        ));
    }
    list.push_str("];\n");

    let mig_out = Path::new(&env::var("OUT_DIR").unwrap()).join("migrations.rs");
    fs::write(&mig_out, list).expect("write migrations.rs");

    println!("cargo:rerun-if-changed=migration");
}
