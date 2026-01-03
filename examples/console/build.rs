fn main() -> anyhow::Result<()> {
    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tlink.x");

    println!("cargo:rustc-link-arg=-Tdefmt.x");

    Ok(())
}
