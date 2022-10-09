fn main() {
    std::fs::read_dir("generator")
        .unwrap()
        .map(|entry| entry.unwrap().path().to_str().unwrap().to_owned())
        .filter(|path| path.ends_with(".ts") || path.ends_with(".ejs"))
        .for_each(|path| println!("cargo:rerun-if-changed={}", path));

    assert!(std::process::Command::new("pwsh")
        .current_dir("generator")
        .arg("-noprofile")
        .arg("-c")
        .arg("ts-node-esm index.ts; exit $LASTEXITCODE")
        .status()
        .unwrap()
        .success());
}
