//! 该模块提供了加载和保存 Quasar Trajectory 配置设置的功能。配置存储在 `Param.toml` 文件中，
//! 可以位于以下描述的各种路径中。
//!
//! # 配置文件搜索顺序
//!
//! 配置文件 `Param.toml` 按以下顺序搜索：
//! 1. 环境变量 `QUASAR_CONFIG_PATH` 指定的路径。
//! 2. 项目根目录（由环境变量 `CARGO_MANIFEST_DIR` 确定）。
//! 3. 命令行的当前工作目录（pwd）。
//!
//! # 结构体
//!
//! - `Config`: 一个 `OnceLock<ConfigInner>` 的包装器，用于提供对配置的全局访问。
//! - `ConfigInner`: 持有实际配置数据的内部结构，包括：
//!   - `camera`: 相机设置的配置。
//!   - `detect`: 检测设置的配置。
//!   - `track`: 跟踪设置的配置。
//!   - `robot`: 机器人设置的配置。
//! - `Camera`: 相机的配置设置，例如曝光和增益。
//! - `Detect`: 检测的配置设置（当前为空）。
//! - `Track`: 跟踪的配置设置（当前为空）。
//! - `Robot`: 机器人的配置设置（当前为空）。
//!
//! # 函数
//!
//! - `find_config() -> Option<(File, PathBuf)>`: 在指定路径中搜索配置文件，如果找到则返回文件句柄和路径。
//! - `load_config() -> ConfigInner`: 从 `Param.toml` 文件加载配置并返回 `ConfigInner` 结构。如果文件未找到或无法读取，则会引发 panic。
//! - `save_config()`: 将当前配置保存到 `Param.toml` 文件。如果文件无法写入，则会引发 panic。
//!
//! # 用法
//!
//! 要访问配置，请使用全局 `CONFIG` 常量。例如：
//!
//! ```rust
//! let camera_config = CONFIG.camera.lock().unwrap();
//! println!("Camera exposure auto: {}", camera_config.exposure_auto);
//! ```
//!
//! 在进行更改后保存配置：
//!
//! ```rust
//! {
//!     let mut camera_config = CONFIG.camera.lock().unwrap();
//!     camera_config.exposure_auto = false;
//! }
//! save_config();
//! ```
use std::{
    fs::File,
    io::Read,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use serde::{Deserialize, Serialize};

pub static CONFIG: Config = Config::new();

pub struct Config(OnceLock<ConfigInner>);

impl Config {
    const fn new() -> Self {
        Self(OnceLock::new())
    }

    pub fn get(&self) -> &ConfigInner {
        self.0.get_or_init(|| load_config())
    }
}

impl Deref for Config {
    type Target = ConfigInner;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl DerefMut for Config {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.get_mut().expect("全局配置未初始化")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigInner {
    pub camera: Mutex<Camera>,
    pub detect: Mutex<Detect>,
    pub track: Mutex<Track>,
    pub robot: Mutex<Robot>,
    pub gui: Mutex<GUI>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct Camera {
    pub exposure_auto: bool,
    pub gain_auto: bool,
    pub exposure_time: f32,
    pub gain: f32,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Detect {}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Track {}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Robot {}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct GUI {}

fn find_config() -> Option<(File, PathBuf)> {
    // 从环境变量获取
    if let Ok(env_path) = std::env::var("QUASAR_CONFIG_PATH") {
        let env_path = PathBuf::from(env_path);
        if let Ok(fp) = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&env_path)
        {
            return Some((fp, env_path));
        } else if let Ok(fp) = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(env_path.join("Param.toml"))
        {
            return Some((fp, env_path.join("Param.toml")));
        }
    }

    // 检查默认路径
    if let Ok(cargo_manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let default_path = PathBuf::from(cargo_manifest_dir).join("Param.toml");
        if let Ok(fp) = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&default_path)
        {
            return Some((fp, default_path));
        }
    }

    // 检查当前目录
    let current_dir = std::env::current_dir().ok()?;
    let current_config = current_dir.join("Param.toml");
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&current_config)
        .ok()
        .map(|fp| (fp, current_config))
}

fn load_config() -> ConfigInner {
    let (mut config, config_path) = find_config().expect(
        "在查找路径上未找到配置文件Param.toml。有三种解决方案，请任选其一：
    1. 请设置环境变量QUASAR_CONFIG_PATH，指向Param.toml的绝对路径
    2. 将Param.toml放置到项目根目录
    3. 将Param.toml放置到命令行中的当前目录（pwd）",
    );
    let mut config_str = String::new();
    config.read_to_string(&mut config_str).unwrap_or_else(|_| {
        panic!(
            "读取指定的Param.toml失败，指定的文件编码包含非合法UTF-8的字节。
Param.toml的读取路径为：{}，若此路径与你实际指定的路径不符，请详细阅读下方说明。
Quasar Trajectory对于配置文件的查找路径有三种方案，按优先级依次排序：
    1. 环境变量QUASAR_CONFIG_PATH所指向的路径
    2. 项目根目录
    3. 命令行执行命令的当前目录（pwd）",
            config_path.display()
        )
    });

    toml::from_str(&config_str).unwrap_or_else(|err| {
        panic!(
            "指定的配置文件“{}”解析失败: {}，请检查此文件的内容是否正确",
            config_path.display(),
            err
        )
    })
}

#[cfg(feature = "gui")]
pub fn save_config() {
    let config = toml::to_string(CONFIG.get()).expect("序列化全局config失败");

    let (_, config_path) = find_config().expect(
        "在查找路径上未找到配置文件Param.toml。有三种解决方案，请任选其一：
    1. 请设置环境变量QUASAR_CONFIG_PATH，指向Param.toml的绝对路径
    2. 将Param.toml放置到项目根目录
    3. 将Param.toml放置到命令行中的当前目录（pwd）",
    );

    std::fs::write(&config_path, config).unwrap_or_else(|_| panic!("Param.toml写入失败，写入路径为：{}，若此路径与你实际指定的路径不符，请详细阅读下方说明。
    Quasar Trajectory对于配置文件的查找路径有三种方案，按优先级依次排序：
    1. 环境变量QUASAR_CONFIG_PATH所指向的路径
    2. 项目根目录
    3. 命令行执行命令的当前目录（pwd）",
        config_path.display()));
}
