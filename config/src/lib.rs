use std::{fs::File, io::Read, path::PathBuf, sync::RwLock};

use serde::{Deserialize, Serialize};

pub static CONFIG: RwLock<Option<Config>> = RwLock::new(None);

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub camera: Camera,
    pub detect: Detect,
    pub track: Track,
    pub robot: Robot,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Camera {
    exposure_time: u64,
    gain: u64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Detect {}
#[derive(Serialize, Deserialize, Debug)]
pub struct Track {}
#[derive(Serialize, Deserialize, Debug)]
pub struct Robot {}

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
            .open(env_path.join("config.toml"))
        {
            return Some((fp, env_path.join("config.toml")));
        }
    }

    // 检查默认路径
    if let Ok(cargo_manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let default_path = PathBuf::from(cargo_manifest_dir).join("config.toml");
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
    let current_config = current_dir.join("config.toml");
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&current_config)
        .ok()
        .map(|fp| (fp, current_config))
}

pub fn load_config() {
    let (mut config, config_path) = find_config().expect(
        "在查找路径上未找到配置文件config.toml。有三种解决方案，请任选其一：
    1. 请设置环境变量QUASAR_CONFIG_PATH，指向config.toml的绝对路径
    2. 将config.toml放置到项目根目录
    3. 将config.toml放置到命令行中的当前目录（pwd）",
    );
    let mut config_str = String::new();
    config.read_to_string(&mut config_str).unwrap_or_else(|_| {
        panic!(
            "读取指定的config.toml失败，指定的文件编码包含非合法UTF-8的字节。
config.toml的读取路径为：{}，若此路径与你实际指定的路径不符，请详细阅读下方说明。
Quasar Trajectory对于配置文件的查找路径有三种方案，按优先级依次排序：
    1. 环境变量QUASAR_CONFIG_PATH所指向的路径
    2. 项目根目录
    3. 命令行执行命令的当前目录（pwd）",
            config_path.display()
        )
    });
    let mut config_lock = CONFIG.write().expect("获取全局config的写锁失败");
    *config_lock = Some(toml::from_str(&config_str).unwrap_or_else(|_| {
        panic!(
            "指定的配置文件“{}”解析失败，请检查此文件的内容是否正确",
            config_path.display()
        )
    }));
}

pub fn save_config() {
    let config_lock = CONFIG.read().expect("获取全局config的读锁失败");
    let config = toml::to_string(config_lock.as_ref().expect("全局config未初始化"))
        .expect("序列化全局config失败");

    let (_, config_path) = find_config().expect(
        "在查找路径上未找到配置文件config.toml。有三种解决方案，请任选其一：
    1. 请设置环境变量QUASAR_CONFIG_PATH，指向config.toml的绝对路径
    2. 将config.toml放置到项目根目录
    3. 将config.toml放置到命令行中的当前目录（pwd）",
    );

    std::fs::write(&config_path, config).unwrap_or_else(|_| panic!("config.toml写入失败，写入路径为：{}，若此路径与你实际指定的路径不符，请详细阅读下方说明。
    Quasar Trajectory对于配置文件的查找路径有三种方案，按优先级依次排序：
    1. 环境变量QUASAR_CONFIG_PATH所指向的路径
    2. 项目根目录
    3. 命令行执行命令的当前目录（pwd）",
        config_path.display()));
}
