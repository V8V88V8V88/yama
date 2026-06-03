fn main() {
    cc::Build::new()
        .file("c_src/file_manager.c")
        .include("include")
        .compile("file_manager");

    println!("cargo:rerun-if-changed=c_src/file_manager.c");
    println!("cargo:rerun-if-changed=include/file_manager.h");
}
