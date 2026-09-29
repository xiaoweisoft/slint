fn main() {
    println!("cargo:rerun-if-changed=owned_data.cpp");
    println!("cargo:rerun-if-changed=native");
    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .include("native")
        .file("owned_data.cpp")
        .compile("skia_owned_data");
}
