use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"),
    );
    let dist_dir = manifest_dir.join("../dist-web");
    let index_file = dist_dir.join("index.html");

    println!("cargo:rerun-if-changed={}", dist_dir.display());

    if !index_file.is_file() {
        panic!(
            "Web frontend is missing at {}. Run `pnpm build:web` before building cc-switch-web.",
            index_file.display()
        );
    }
}
