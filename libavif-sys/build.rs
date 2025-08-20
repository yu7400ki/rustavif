/*
Copyright 2020 Charles Samuels <ks@ks.ax>, Paolo Barbolini <paolo@paolo565.org>

Redistribution and use in source and binary forms, with or without modification,
are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
this list of conditions and the following disclaimer in the documentation and/or
other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/
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

    #[cfg(feature = "codec-rav1e")]
    {
        use std::fs;

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let rav1e_inc = out_dir.join("include").join("rav1e");
        fs::create_dir_all(&rav1e_inc).expect("mkdir rav1e include dir failed");

        let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        fs::copy(crate_dir.join("rav1e.h"), rav1e_inc.join("rav1e.h"))
            .expect("copy rav1e.h failed");

        cfg.define("AVIF_CODEC_RAV1E", "SYSTEM")
            .define("AVIF_CODEC_LIBRARIES", "rav1e")
            .define("RAV1E_INCLUDE_DIR", &rav1e_inc)
            .define("RAV1E_LIBRARY", "-rav1e");
    }
    #[cfg(not(feature = "codec-rav1e"))]
    {
        cfg.define("AVIF_CODEC_RAV1E", "OFF");
    }

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
