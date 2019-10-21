use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

use autotools;

fn get_sysroot(sdk_name: &str) -> Result<String, Box<dyn Error>> {
    let mut cmd = Command::new("xcrun");
    cmd.args(&["--sdk", sdk_name, "--show-sdk-path"]);
    Ok(String::from_utf8(cmd.output()?.stdout)?)
}

fn ios_target_config(
    config: &mut autotools::Config,
    target_triple: &str,
) -> Result<(), Box<dyn Error>> {
    let sysroot = match target_triple {
        "aarch64-apple-ios" => get_sysroot("iphoneos")?,
        "x86_64-apple-ios" => get_sysroot("iphonesimulator")?,
        _ => panic!(
            "unsupported target triple (`{}`) while building for iOS",
            target_triple
        ),
    };

    config
        .cxxflag(format!("-isysroot {}", sysroot))
        .ldflag(format!("-isysroot {}", sysroot));

    Ok(())
}

fn android_target_config(
    config: &mut autotools::Config,
    target_triple: &str,
) -> Result<(), Box<dyn Error>> {
    let cc_var = format!("CARGO_TARGET_{}_CC", target_triple.to_ascii_uppercase().replace("-", "_"));
    let cc = env::var(cc_var).expect("CARGO_TARGET_<triplet>_CC needs to be set");
    let cxx = format!("{}++", cc);
    config.env("CXX", cxx);
    config.env("CC", cc);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let target_os =
        env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS should always be set");

    let target_triple = env::var("TARGET").expect("TARGET should always be set");

    let mut builder = bindgen::Builder::default();

    let libcpp = match target_os.as_ref() {
        "macos" | "ios" => Some("dylib=c++"),
        "linux" => Some("dylib=stdc++"),
        _ => None,
    };

    if pkg_config::probe_library("hunspell").is_err() {
        let mut autoconf = autotools::Config::new("vendor");

        autoconf.reconf("-iv").cxxflag("-fPIC");

        // We need to pass `--host` explicitly to the configure script. There is some confusion
        // going on in the `autotools` crate about these options: what we call `host` and `target`
        // correspond to `build` and `host` respectively in autoconf.
        // See: https://www.gnu.org/software/autoconf/manual/autoconf-2.65/html_node/Specifying-Target-Triplets.html
        autoconf.host(&target_triple);

        // We need to explicitly enable the output archs we want, even though we are specifying
        // `--host`.
        if target_triple.starts_with("aarch64-") {
            autoconf.cxxflag("-arch arm64").ldflag("-arch arm64");
        }

        if target_os == "ios" {
            ios_target_config(&mut autoconf, &target_triple)?;
        }

        if target_os == "android" {
            android_target_config(&mut autoconf, &target_triple)?;
        }

        // Avoid building the tools subdirectory. We don't need it, as we only need libhunspell,
        // and it won't compile when targeting the `iphoneos` sdk as the tools use functions
        // (like `system()`) that are not available there.
        autoconf.make_args(vec!["-C".to_owned(), "src/hunspell".to_owned()]);

        let dst = autoconf.build();

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
