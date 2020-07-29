use bindgen;

use cmake::Config;
use std::env;
use std::fs;
use std::path::PathBuf;

fn build_slang() {
    let dst = Config::new("slang")
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("SLANG_INCLUDE_TESTS", "OFF")
        .define(
            "CMAKE_INSTALL_PREFIX",
            format!("{}/slang_wrapper", env::var("CARGO_MANIFEST_DIR").unwrap()),
        )
        .build();
    println!("{}", dst.display());
}

fn build_slang_wrapper(fmt_include_dir: &str) {
    cc::Build::new()
        .cpp(true)
        .flag("-std=c++17")
        .static_flag(true)
        .include("slang_wrapper/include")
        .include(fmt_include_dir)
        .file("slang_wrapper/src/slang_lib.cpp")
        .file("slang_wrapper/src/basic_client.cpp")
        .out_dir("slang_wrapper/lib")
        .compile("slangwrapper");
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rerun-if-changed=slang_wrapper/src/slang_wrapper.h");
    println!("cargo:rerun-if-changed=slang_wrapper/src/slang_lib.cpp");

    build_slang();

    let conan_build_info =
        fs::read_to_string(format!("{}/build/conanbuildinfo.txt", out_dir)).unwrap();
    let mut lines = conan_build_info.lines();
    while lines.next().unwrap() != "[libdirs_fmt]" {}
    let fmt_lib_dir = lines.next().unwrap();
    lines = conan_build_info.lines();
    while lines.next().unwrap() != "[includedirs_fmt]" {}
    let fmt_include_dir = lines.next().unwrap();

    build_slang_wrapper(fmt_include_dir);

    let bindings = bindgen::Builder::default()
        .clang_arg("-x")
        .clang_arg("c++")
        .header("slang_wrapper/src/slang_wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    println!(
        "cargo:rustc-link-search=native={}/slang_wrapper/lib",
        env::var("CARGO_MANIFEST_DIR").unwrap()
    );
    println!("cargo:rustc-link-search=native={}", fmt_lib_dir);
    println!("cargo:rustc-link-search=native=/usr/lib");

    println!("cargo:rustc-link-lib=static=slangwrapper");
    println!("cargo:rustc-link-lib=static=slangcompiler");
    println!("cargo:rustc-link-lib=static=slangruntime");
    println!("cargo:rustc-link-lib=static=slangparser");
    println!("cargo:rustc-link-lib=static=slangcore");
    println!("cargo:rustc-link-lib=static=fmt");
    println!("cargo:rustc-link-lib=static=stdc++");

    let out_path = PathBuf::from(out_dir);
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
