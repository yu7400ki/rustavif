use std::env;
use std::path::PathBuf;

fn main() {
    let mut cfg = cmake::Config::new("libavif");

    cfg.profile(match env::var("PROFILE").as_deref() {
        Ok("release") => "Release",
        _ => "Debug",
    });
    cfg.define("AVIF_BUILD_APPS", "OFF")
        .define("AVIF_BUILD_TESTS", "OFF")
        .define("AVIF_ENABLE_COVERAGE", "OFF")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .define("BUILD_SHARED_LIBS", "OFF");

    if cfg!(target_env = "msvc") {
        cfg.define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreaded");
    }

    cfg.define(
        "AVIF_LIBYUV",
        if cfg!(feature = "libyuv") {
            "LOCAL"
        } else {
            "OFF"
        },
    );

    cfg.define(
        "AVIF_CODEC_AOM",
        if cfg!(feature = "codec-aom") {
            "LOCAL"
        } else {
            "OFF"
        },
    );
    cfg.define(
        "AVIF_CODEC_DAV1D",
        if cfg!(feature = "codec-dav1d") {
            "LOCAL"
        } else {
            "OFF"
        },
    );
    cfg.define(
        "AVIF_CODEC_LIBGAV1",
        if cfg!(feature = "codec-gav1") {
            "LOCAL"
        } else {
            "OFF"
        },
    );
    cfg.define(
        "AVIF_CODEC_RAV1E",
        if cfg!(feature = "codec-rav1e") {
            "LOCAL"
        } else {
            "OFF"
        },
    );
    cfg.define(
        "AVIF_CODEC_SVT",
        if cfg!(feature = "codec-svt") {
            "LOCAL"
        } else {
            "OFF"
        },
    );

    let dst = cfg.build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());

    println!("cargo:rustc-link-lib=static=avif");

    #[cfg(target_family = "unix")]
    {
        println!("cargo:rustc-link-lib=m");
        println!("cargo:rustc-link-lib=pthread");
    }

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("libavif/include/avif/avif.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
