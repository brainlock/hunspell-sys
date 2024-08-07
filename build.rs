use std::env;
use std::error::Error;
use std::path::PathBuf;

fn build_static_hunspell() {
    let mut cpp_build = cc::Build::new();

    cpp_build
        .file("vendor/src/hunspell/affentry.cxx")
        .file("vendor/src/hunspell/affixmgr.cxx")
        .file("vendor/src/hunspell/csutil.cxx")
        .file("vendor/src/hunspell/filemgr.cxx")
        .file("vendor/src/hunspell/hashmgr.cxx")
        .file("vendor/src/hunspell/hunspell.cxx")
        .file("vendor/src/hunspell/hunzip.cxx")
        .file("vendor/src/hunspell/phonet.cxx")
        .file("vendor/src/hunspell/replist.cxx")
        .file("vendor/src/hunspell/suggestmgr.cxx")
        .define("BUILDING_LIBHUNSPELL", "1")
        .cpp(true)
        .flag("-std=c++11");

    cpp_build.compile("hunspell-1.7");

    println!("cargo:rustc-link-lib=static=hunspell-1.7");
}

fn generate_bindings_headers() -> Result<(), Box<dyn Error>> {
    let target_triple = env::var("TARGET").expect("TARGET should always be set");

    let mut builder = bindgen::Builder::default();

    builder = builder.clang_arg(format!("-I{}", "vendor/src"));

    if target_triple == "aarch64-apple-ios" {
        // I say aarch64, you say arm64...
        builder = builder.clang_arg("--target=arm64-apple-ios");
    }

    let bindings = builder
        .header("wrapper.h")
        .generate()
        .expect("could not generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR")?);
    bindings.write_to_file(out_path.join("bindings.rs"))?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    build_static_hunspell();
    generate_bindings_headers()?;
    Ok(())
}
