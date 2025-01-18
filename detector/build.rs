use std::{env, path::PathBuf};

fn main() {
    if cfg!(feature = "hikvision") {
        let lib = PathBuf::from(env::var("MVCAM_COMMON_RUNENV")
        .expect("未设置环境变量 MVCAM_COMMON_RUNENV，该环境变量应当指向海康威视的相机驱动下的lib目录，一般情况下目录位置位于/opt/MVS/lib")).join("aarch64");
        let include = PathBuf::from(env::var("MVCAM_SDK_PATH").expect("未设置环境变量 MVCAM_SDK_PATH，该环境变量应当指向海康威视的相机驱动目录，一般情况下目录位置位于/opt/MVS")).join("include");

        println!("cargo:rustc-link-search=native={}", lib.display());
        println!("cargo:rustc-link-arg=-Wl,-rpath={}", lib.display());

        println!("cargo:rustc-link-lib=dylib=MvCameraControl");

        cc::Build::new()
            .file("camera/hikvision/lib.c")
            .include(include)
            .compile("hikcamera");
    } else if cfg!(feature = "mindvision") {
        // TO DO
    } else {
        panic!("请在features选项中指定相机品牌相应的开发包");
    }

    let name = if cfg!(feature = "hikvision") {
        "hikvision"
    } else if cfg!(feature = "mindvision") {
        "mindvision"
    } else {
        panic!()
    };

    let bindings = bindgen::Builder::default()
        .header(format!("camera/{name}/lib.h"))
        .generate()
        .expect("生成相机的 C API binding 时出错！");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("camera.rs")).expect(
        format!(
            "向文件 \"{}\" 写入 C API binding 时出错！",
            out_path.join("camera.rs").display(),
        )
        .as_str(),
    );
}
