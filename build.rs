fn main() {
    println!("cargo:rerun-if-changed=generator/index.ts");
    println!("cargo:rerun-if-changed=generator/extra-schemas.ts");
    println!("cargo:rerun-if-changed=generator/server.ejs");
    assert!(std::process::Command::new("pwsh")
        .current_dir("generator")
        .arg("-noprofile")
        .arg("-c")
        .arg("ts-node-esm index.ts; exit $LASTEXITCODE")
        .status()
        .unwrap()
        .success());
}
