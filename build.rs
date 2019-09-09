use std::env;
use std::error::Error;
use std::path::PathBuf;

use autotools;

fn main() -> Result<(), Box<dyn Error>> {
    let target_os =
        env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS should always be set");

    let mut builder = bindgen::Builder::default();

    let libcpp = match target_os.as_ref() {
        "macos" => Some("dylib=c++"),
        "linux" => Some("dylib=stdc++"),
        _ => None,
    };

    if pkg_config::probe_library("hunspell").is_err() {
        let dst = autotools::Config::new("vendor")
            .reconf("-ivf")
            .cxxflag("-fPIC")
            .build();

        println!(
            "cargo:rustc-link-search=native={}",
            dst.join("lib").display()
        );
        println!("cargo:rustc-link-lib=static=hunspell-1.7");

        if let Some(link) = libcpp {
            println!("cargo:rustc-link-lib={}", link);
        }

        builder = builder.clang_arg(format!("-I{}", dst.join("include").display()));
    }

    let bindings = builder
        .header("wrapper.h")
        .generate()
        .expect("could not generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR")?);
    bindings.write_to_file(out_path.join("bindings.rs"))?;

    Ok(())
}
