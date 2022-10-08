fn main() {
    println!("cargo:rerun-if-changed=generator/index.ts");
    println!("cargo:rerun-if-changed=generator/extra-schemas.ts");
    std::process::Command::new("pwsh")
        .arg("/c")
        .arg("ts-node-esm")
        .arg("generator/index.ts")
        .status()
        .unwrap();
}
