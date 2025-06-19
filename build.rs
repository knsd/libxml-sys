fn main() {
    let target = std::env::var("TARGET").expect("TARGET was not set");
    let mut cfg = cmake::Config::new("vendor/libxml2");

    cfg.define("BUILD_SHARED_LIBS", "OFF")
        .define("LIBXML2_WITH_LZMA", "OFF")
        .define("LIBXML2_WITH_ICONV", "OFF")
        .define("LIBXML2_WITH_ZLIB", "OFF")
        .define("LIBXML2_WITH_PYTHON", "OFF")
        .define("LIBXML2_WITH_TESTS", "OFF")
        .define("LIBXML2_WITH_FTP", "OFF")
        .define("LIBXML2_WITH_HTTP", "OFF")
        .define("LIBXML2_WITH_PROGRAMS", "OFF");

    if target == "wasm32-wasip1" {
        cfg.define("LIBXML2_WITH_THREADS", "OFF");
        cfg.define("HAVE_PTHREAD_H", "OFF");
        let sdk_path = std::env::var("WASI_SDK_PATH").unwrap_or_else(|_| "/opt/wasi-sdk".into());
        cfg.define(
            "CMAKE_TOOLCHAIN_FILE",
            format!("{}/share/cmake/wasi-sdk.cmake", sdk_path),
        );
    } else {
        cfg.define("LIBXML2_WITH_THREADS", "ON");
    }

    let dst = cfg.build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=xml2");
    println!("cargo:include={}/include", dst.display());
}
