use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process;

use autotools;

fn run() -> Result<(), Box<Error>> {
    let target = std::env::var("TARGET").unwrap();
    let mut builder = bindgen::Builder::default();

    let cpp_link = if target.contains("linux"){
        "dylib=stdc++"
    }else if target.contains("-apple-"){
        "dylib=c++"
    }else {
        panic!("Unsupported target (for now?)");
    };

    if pkg_config::probe_library("hunspell").is_err() {
        let dst = autotools::Config::new("vendor")
            .reconf("-ivf")
            .cxxflag("-fPIC")
            .build();

        println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
        println!("cargo:rustc-link-lib=static=hunspell-1.7");

        println!("cargo:rustc-link-lib={}", cpp_link);

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

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}
