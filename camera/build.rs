use std::{
    env, fs,
    path::{Path, PathBuf},
};
fn main() {
    if cfg!(feature = "hikvision") {
        //  海康include目录
        let include = PathBuf::from(env::var("MVCAM_SDK_PATH").expect("未设置环境变量 MVCAM_SDK_PATH，该环境变量应当指向海康威视的相机驱动目录，一般情况下目录位置位于`/opt/MVS`")).join("include");
        if !include.exists() {
            panic!("未找到海康威视相机驱动的include目录，请检查MVCAM_SDK_PATH环境变量是否设置正确");
        }
        let src_path = Path::new("src_c/hikvision");
        if !src_path.exists() {
            panic!("未找到相机驱动调用的C源代码，请检查src_c/hikvision目录是否存在");
        }
        // 源代码变更检测，告知编译器在下述路径的源代码发生变更时重新编译
        println!("cargo:rerun-if-changed={}", src_path.display());

        // C Compiler 编译海康威视相机驱动的C API
        cc::Build::new()
            .file(src_path.join("api.c"))
            .include(&include)
            .flag("-std=c2x")
            .compile("hikcamera");

        // 获取环境变量，获取海康威视相机驱动的lib目录
        let lib = PathBuf::from(env::var("MVCAM_COMMON_RUNENV")
        .expect("未设置环境变量 MVCAM_COMMON_RUNENV，该环境变量应当指向海康威视的相机驱动下的lib目录，一般情况下目录位置位于/opt/MVS/lib")).join("aarch64");
        if !lib.exists() {
            panic!(
                "未找到海康威视相机驱动的lib目录，请检查MVCAM_COMMON_RUNENV环境变量是否设置正确"
            );
        }
        if !lib.join("libMvCameraControl.so").exists() {
            panic!("未找到海康威视相机驱动的libMvCameraControl.so，请检查MVCAM_COMMON_RUNENV环境变量是否设置正确");
        }
        // 通知链接器链接海康威视相机驱动
        println!("cargo:rustc-link-search=native={}", lib.display());
        // 链接器选项
        println!("cargo:rustc-link-arg=-Wl,-rpath={}", lib.display());
        // 链接库MvCameraControl
        println!("cargo:rustc-link-lib=dylib=MvCameraControl");

        let name = if cfg!(feature = "hikvision") {
            "hikvision"
        } else if cfg!(feature = "mindvision") {
            "mindvision"
        } else {
            panic!("请在features选项中指定相机品牌相应的开发包");
        };

        // 使用 bindgen 生成 Rust 的 C API binding
        let bindings = bindgen::Builder::default()
            .header(src_path.join("api.h").to_str().unwrap())
            // .clang_arg(format!("-I{}", include.display()))
            .generate_comments(true)
            .clang_arg("-fparse-all-comments")
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
        println!("cargo:rerun-if-changed=src_c/mindvision");
    } else {
        panic!("请在features选项中指定相机品牌相应的开发包");
    }
}
