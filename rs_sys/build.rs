use std::env;
use std::path::PathBuf;

fn main() {
    let dbcppp = "../dbcppp";
    let dst = cmake::Config::new(dbcppp)
        .define("BUILD_STATIC_LIBS", "ON")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_TESTING", "OFF")
        .cxxflag("-lstdc++")
        .always_configure(true)
        .build();
    println!("cargo:rustc-link-search=native={}/build/src/libdbcppp", dst.display());
    println!("cargo:rustc-link-lib=static=dbcppp");
    println!("cargo:rustc-link-arg=-lstdc++");

    let header = format!("{}/include/dbcppp/CApi.h", dbcppp);
    println!("cargo:rerun-if-changed={}", header);
    let bindings = bindgen::Builder::default()
        .header(header)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
