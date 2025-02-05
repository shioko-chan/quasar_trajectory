use std::{env, fs, path::PathBuf};
fn main() {
    if cfg!(feature = "hikvision") {
        // 获取环境变量，获取海康威视相机驱动的lib目录
        let lib = PathBuf::from(env::var("MVCAM_COMMON_RUNENV")
        .expect("未设置环境变量 MVCAM_COMMON_RUNENV，该环境变量应当指向海康威视的相机驱动下的lib目录，一般情况下目录位置位于/opt/MVS/lib")).join("aarch64");
        //  海康include目录
        let include = PathBuf::from(env::var("MVCAM_SDK_PATH").expect("未设置环境变量 MVCAM_SDK_PATH，该环境变量应当指向海康威视的相机驱动目录，一般情况下目录位置位于`/opt/MVS`")).join("include");

        // 源代码变更检测，告知编译器在下述路径的源代码发生变更时重新编译
        println!("cargo:rerun-if-changed=camera/hikvision");

        // 通知链接器链接海康威视相机驱动
        println!("cargo:rustc-link-search=native={}", lib.display());
        // 链接器选项
        println!("cargo:rustc-link-arg=-Wl,-rpath={}", lib.display());
        // 链接静态库MvCameraControl
        println!("cargo:rustc-link-lib=dylib=MvCameraControl");

        // C Compiler 编译海康威视相机驱动的C API
        cc::Build::new()
            .file("camera/hikvision/api.c")
            .include(&include)
            .compile("hikcamera");

        let name = if cfg!(feature = "hikvision") {
            "hikvision"
        } else if cfg!(feature = "mindvision") {
            "mindvision"
        } else {
            panic!("请在features选项中指定相机品牌相应的开发包");
        };
        // 使用 bindgen 生成 Rust 的 C API binding
        let bindings = bindgen::Builder::default()
            .header(format!("camera/{name}/api.h"))
            // .clang_arg(format!("-I{}", include.display()))
            .generate_comments(true)
            .generate()
            .expect("生成相机的 C API binding 时出错！");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("hikcamera");
        fs::create_dir_all(&out_path).expect("创建目录时出错！");
        bindings
            .write_to_file(out_path.join("camera.rs"))
            .expect("写入相机的 C API binding 时出错！");
    } else if cfg!(feature = "mindvision") {
        // TO DO

        // 源代码变更检测
        println!("cargo:rerun-if-changed=camera/hikvision");
    } else {
        panic!("请在features选项中指定相机品牌相应的开发包");
    }
}
