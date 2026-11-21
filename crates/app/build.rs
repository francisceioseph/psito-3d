//! Embeds the icon and the version in the Windows executable. Other
//! systems take the icon from the package (Linux) or need nothing here.

fn main() {
    println!("cargo:rerun-if-changed=../../assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/icon.ico");
        res.set("ProductName", "Psito 3D");
        res.set("FileDescription", "Psito 3D");
        res.set("LegalCopyright", "Psito 3D contributors, Apache License 2.0");
        if let Err(e) = res.compile() {
            panic!("could not embed the Windows resources: {e}");
        }
    }
}
