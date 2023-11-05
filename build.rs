use std::env;
use std::path::PathBuf;

fn main() {
    // see https://github.com/evilsocket/legba/issues/8
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "macos" {
        let homebrew_prefix =
            PathBuf::from(env::var("HOMEBREW_PREFIX").unwrap_or("/opt/homebrew".into()));

        let lib_dir = homebrew_prefix.join("lib");
        let include_dir = homebrew_prefix.join("include");

        // Tell cargo to link against libcurl from Homebrew
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=curl");
        println!("cargo:include={}", include_dir.display());
    }
}
