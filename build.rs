fn main() {
    println!("cargo:rerun-if-changed=generator/index.ts");
    println!("cargo:rerun-if-changed=generator/extra-schemas.ts");
    println!("cargo:rerun-if-changed=generator/server.ejs");
    assert!(std::process::Command::new("pwsh")
        .arg("/c")
        .current_dir("generator")
        .arg("ts-node-esm")
        .arg("index.ts")
        .status()
        .unwrap()
        .success());
}
