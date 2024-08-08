use std::{env, path::PathBuf};

fn main() {
    use build_script_cfg::Cfg;
    use search_mx_tools::find_mx_home;

    println!("cargo:rereun-if-changed=build.rs");

    let mx = Cfg::new("detected_mx");
    let Some(mx_home) = find_mx_home() else {
        return;
    };
    mx.define();
    println!("{}", mx_home.join("lib").display());
    println!(
        "cargo:rustc-link-search=native={}",
        mx_home.join("lib").display()
    );
    println!("cargo:rustc-link-lib=dylib=mcruntime");
    println!("cargo:rustc-link-lib=dylib=mxc-runtime64");

    println!("cargo-rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", mx_home.join("include").display()))
        .allowlist_item("mc.*")
        .must_use_type("mcError_t")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
