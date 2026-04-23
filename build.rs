//! Compiles `palettes/*.toml` into a Rust source file that defines one
//! `pub const <SLUG>: Palette` per file and collects them into
//! `BUILTIN_PALETTES` in alphabetical (filename) order.

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
struct PaletteFile {
    name: String,
    #[serde(default)]
    credit: Option<String>,
    stops: Vec<[u16; 3]>,
}

fn main() {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR unset");
    let palettes_dir = Path::new(&manifest_dir).join("palettes");
    println!("cargo:rerun-if-changed=palettes");

    let mut entries: Vec<PathBuf> = fs::read_dir(&palettes_dir)
        .unwrap_or_else(|e| panic!("reading {}: {e}", palettes_dir.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension() == Some(OsStr::new("toml")))
        .collect();
    entries.sort();

    assert!(
        !entries.is_empty(),
        "no palette files found under {}",
        palettes_dir.display()
    );

    let mut out = String::new();
    out.push_str("// GENERATED FILE. Do not edit. See build.rs and palettes/*.toml.\n\n");

    let mut slugs: Vec<String> = Vec::with_capacity(entries.len());
    let mut names: Vec<String> = Vec::with_capacity(entries.len());

    for path in &entries {
        let slug_raw = path
            .file_stem()
            .and_then(OsStr::to_str)
            .expect("palette filename not utf-8");
        let slug = slug_raw.to_ascii_uppercase();

        let text =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
        let pf: PaletteFile = toml::from_str(&text)
            .unwrap_or_else(|e| panic!("{}: invalid TOML: {e}", path.display()));

        assert_eq!(
            pf.stops.len(),
            16,
            "{}: expected 16 stops, got {}",
            path.display(),
            pf.stops.len()
        );

        out.push_str(&format!("/// {} palette.", pf.name));
        if let Some(c) = pf.credit.as_deref() {
            out.push_str(&format!("\n///\n/// {c}"));
        }
        out.push_str(&format!("\npub const {slug}: Palette = [\n"));
        for (i, stop) in pf.stops.iter().enumerate() {
            let [h, s, v] = *stop;
            for (field, val) in [("h", h), ("s", s), ("v", v)] {
                assert!(
                    val <= 255,
                    "{}: stop {i} {field} = {val} out of u8 range",
                    path.display()
                );
            }
            out.push_str(&format!(
                "    hsv16({:3}, {:3}, {:3}),\n",
                h as u8, s as u8, v as u8
            ));
        }
        out.push_str("];\n\n");

        slugs.push(slug);
        names.push(pf.name);
    }

    out.push_str("/// Numeric indices for the built-in palettes. The order matches\n");
    out.push_str("/// [`BUILTIN_PALETTES`] (alphabetical by source filename).\n");
    out.push_str("pub mod id {\n");
    for (i, slug) in slugs.iter().enumerate() {
        out.push_str(&format!("    pub const {slug}: usize = {i};\n"));
    }
    out.push_str("}\n\n");

    out.push_str(&format!(
        "/// All {n} built-in palettes, addressable by the constants in [`id`].\n",
        n = slugs.len()
    ));
    out.push_str(&format!(
        "pub const BUILTIN_PALETTES: [&Palette; {n}] = [\n",
        n = slugs.len()
    ));
    for slug in &slugs {
        out.push_str(&format!("    &{slug},\n"));
    }
    out.push_str("];\n\n");

    out.push_str(
        "/// Human-readable names matching the [`BUILTIN_PALETTES`] order.\n\
         pub const BUILTIN_PALETTE_NAMES: [&str; ",
    );
    out.push_str(&format!("{}] = [\n", names.len()));
    for name in &names {
        out.push_str(&format!("    \"{}\",\n", name.replace('"', "\\\"")));
    }
    out.push_str("];\n");

    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR unset");
    let dest = Path::new(&out_dir).join("palettes_generated.rs");
    fs::write(&dest, out).unwrap_or_else(|e| panic!("writing {}: {e}", dest.display()));
}
