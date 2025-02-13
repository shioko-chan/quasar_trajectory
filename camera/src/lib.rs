#[allow(non_upper_case_globals, non_camel_case_types, non_snake_case, unused)]
mod camera {
    #[cfg(feature = "hikvision")]
    include!(concat!(env!("OUT_DIR"), "/hikcamera/camera.rs"));

    #[cfg(feature = "mindvision")]
    include!(concat!(env!("OUT_DIR"), "/mindcamera/camera.rs"));
}
use anyhow::{ensure, Context, Result};
use camera::*;
// use config::CONFIG;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    ffi::CString,
    fs,
    path::Path,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
    thread,
};
// use utility::ensure_or_stop;

// struct CameraThreadsHandle(Vec<thread::JoinHandle<()>>);

// impl CameraThreadsHandle {
//     fn new(stop_signal: Arc<AtomicBool>, camera_number: u32) -> Result<Self> {
//         init_sdk()?;
//         init_cameras(camera_number)?;
//         Ok((0..camera_number)
//             .map(|cam_id| {
//                 let stop_signal = stop_signal.clone();
//                 thread::spawn(move || {
//                     ensure_or_stop!(config_camera(cam_id).is_ok(), "[cam err01] 相机配置失败");
//                     // stop_signal.load(atomic::Ordering::Relaxed)
//                 })
//             })
//             .collect())
//     }
// }

// impl Drop for CameraThreadsHandle {
//     fn drop(&mut self) {
//         for handle in self.0.drain(..) {
//             if let Err(e) = handle.join() {
//                 warn!("[cam err02] 相机线程退出失败: {:?}", e);
//             }
//         }
//     }
// }

// type Visual = HashMap<String, String>;

// static CONFIG: OnceLock = OnceLock::new();
// pub enum ParamType {
//     Int(IntParamInfo),
//     Float(FloatParamInfo),
//     String(StringParamInfo),
//     Enum(EnumStringList),
//     Bool(bool),
// }

// pub struct ReadonlyParam {
//     pub name: String,
//     pub info: ParamType,
//     pub push_frequency: Option<i32>,
// }

// pub struct EditableParam {
//     pub name: String,
//     pub info: ParamType,
// }

// pub fn load_params_definition<P>(path: P) -> Result<BTreeMap<String, Param>>
// where
//     P: AsRef<Path>,
// {
//     let mut params = BTreeMap::new();
//     let content = fs::read_to_string(path)?;
//     let value: toml::Value = content.parse()?;
//     if let toml::Value::Table(table) = value {
//         for (key, value) in table {
//             if let toml::Value::String(value) = value {
//                 params.insert(key, value);
//             }
//         }
//     }
//     Ok(params)
// }

/// EnumStringList 结构体，用于表示枚举类型的当前值和支持的枚举项
#[derive(Debug)]
pub struct EnumStringList {
    pub current: String,
    pub supported: Vec<String>,
}

impl Into<EnumStringList> for &camera::CEnumStringList {
    fn into(self) -> EnumStringList {
        let current = unsafe { std::ffi::CStr::from_ptr(self.current) }
            .to_str()
            .unwrap_or_default()
            .to_string();

        let supported = (0..self.count)
            .map(|i| unsafe {
                std::ffi::CStr::from_ptr(*self.supported.offset(i as isize))
                    .to_str()
                    .unwrap_or_default()
                    .to_string()
            })
            .collect();

        EnumStringList { current, supported }
    }
}

/// IntParamInfo 结构体，用于表示整型参数的当前值、最小值、最大值和增量
pub type IntParamInfo = CIntParamInfo;

/// FloatParamInfo 结构体，用于表示浮点型参数的当前值、最小值和最大值
pub type FloatParamInfo = CFloatParamInfo;

/// StringParamInfo 结构体，用于表示字符串型参数的当前值和最大长度
#[derive(Debug, Clone)]
pub struct StringParamInfo {
    pub current: String,
    pub max_length: i64,
}

impl Into<StringParamInfo> for &CStringParamInfo {
    fn into(self) -> StringParamInfo {
        let current = unsafe { std::ffi::CStr::from_ptr(self.current) }
            .to_str()
            .unwrap_or_default()
            .to_string();

        StringParamInfo {
            current,
            max_length: self.maxLength,
        }
    }
}

fn set_enum_param_safe(cam_id: u32, name: String, value: String) -> anyhow::Result<()> {
    let ret = unsafe {
        set_enum_param(
            cam_id,
            CString::new(name)?.as_ptr(),
            CString::new(value)?.as_ptr(),
        )
    };
    ensure!(ret.code == 0, "设置枚举参数失败；{}", stringify_err(ret));
    Ok(())
}

fn set_float_param_safe(cam_id: u32, name: String, value: f32) -> anyhow::Result<()> {
    let ret = unsafe { set_float_param(cam_id, CString::new(name)?.as_ptr(), value) };
    ensure!(ret.code == 0, "设置浮点参数失败；{}", stringify_err(ret));
    Ok(())
}

fn set_int_param_safe(cam_id: u32, name: String, value: u32) -> anyhow::Result<()> {
    let ret = unsafe { set_int_param(cam_id, CString::new(name)?.as_ptr(), value) };
    ensure!(ret.code == 0, "设置整型参数失败；{}", stringify_err(ret));
    Ok(())
}

fn set_bool_param_safe(cam_id: u32, name: String, value: bool) -> anyhow::Result<()> {
    let ret = unsafe { set_bool_param(cam_id, CString::new(name)?.as_ptr(), value.into()) };
    ensure!(ret.code == 0, "设置布尔参数失败；{}", stringify_err(ret));
    Ok(())
}

fn set_string_param_safe(cam_id: u32, name: String, value: String) -> anyhow::Result<()> {
    let ret = unsafe {
        set_string_param(
            cam_id,
            CString::new(name)?.as_ptr(),
            CString::new(value)?.as_ptr(),
        )
    };
    ensure!(ret.code == 0, "设置字符串参数失败；{}", stringify_err(ret));
    Ok(())
}

fn stringify_err(ret: APIError) -> String {
    format!(
        "错误来源：{}， 错误码：{:X}",
        if ret.is_thirdparty_err == 1 {
            "`第三方相机SDK`"
        } else {
            "`相机API`"
        },
        ret.code as usize
    )
}

fn get_enum_param_safe(cam_id: u32, name: String) -> anyhow::Result<EnumStringList> {
    let mut info = CEnumStringList {
        current: std::ptr::null_mut(),
        supported: std::ptr::null_mut(),
        count: 0,
    };
    let ret = unsafe {
        get_enum_param(
            cam_id,
            CString::new(name)?.as_ptr(),
            &mut info as *mut CEnumStringList,
        )
    };
    ensure!(ret.code == 0, "获取枚举参数失败；{}", stringify_err(ret));
    Ok((&info).into())
}

fn get_float_param_safe(cam_id: u32, name: String) -> anyhow::Result<FloatParamInfo> {
    let mut info = CFloatParamInfo {
        current: 0.0,
        min: 0.0,
        max: 0.0,
    };
    let ret = unsafe {
        get_float_param(
            cam_id,
            CString::new(name)?.as_ptr(),
            &mut info as *mut CFloatParamInfo,
        )
    };
    ensure!(ret.code == 0, "获取浮点参数失败；{}", stringify_err(ret));
    Ok(info)
}

fn get_int_param_safe(cam_id: u32, name: String) -> anyhow::Result<IntParamInfo> {
    let mut info = CIntParamInfo {
        current: 0,
        min: 0,
        max: 0,
        inc: 0,
    };
    let ret = unsafe {
        get_int_param(
            cam_id,
            CString::new(name)?.as_ptr(),
            &mut info as *mut CIntParamInfo,
        )
    };
    ensure!(ret.code == 0, "获取整型参数失败；{}", stringify_err(ret));
    Ok(info)
}

fn get_string_param_safe(cam_id: u32, name: String) -> anyhow::Result<StringParamInfo> {
    let mut info = CStringParamInfo {
        current: std::ptr::null_mut(),
        maxLength: 0,
    };
    let ret = unsafe {
        get_string_param(
            cam_id,
            CString::new(name)?.as_ptr(),
            &mut info as *mut CStringParamInfo,
        )
    };
    ensure!(ret.code == 0, "获取字符串参数失败；{}", stringify_err(ret));
    Ok((&info).into())
}

fn get_bool_param_safe(cam_id: u32, name: String) -> anyhow::Result<bool> {
    let mut info: u8 = 0;
    let ret = unsafe { get_bool_param(cam_id, CString::new(name)?.as_ptr(), &mut info as *mut u8) };
    ensure!(ret.code == 0, "获取布尔参数失败；{}", stringify_err(ret));
    Ok(info != 0)
}

// 定义参数范围和参数值枚举
#[derive(Debug)]
struct Range<T> {
    min: T,
    max: T,
}

#[derive(Debug)]
enum ParamValue {
    Int {
        val: i64,
        range: Range<i64>,
        inc: i64,
    },
    Float {
        val: f32,
        range: Range<f32>,
    },
    String {
        val: String,
        max_length: i64,
    },
    Enum {
        val: String,
        supported: Vec<String>,
    },
    Bool {
        val: bool,
    },
}

// 实现参数查询函数
fn query_parameters(cam_id: u32, groups: &[Group]) -> Result<BTreeMap<String, ParamValue>> {
    let mut param_table = BTreeMap::new();
    for group in groups {
        for param in &group.parameters {
            let param_name = param.param_ref.clone();
            let param_type = param.param_type.as_str();
            param_table.insert(
                param_name.clone(),
                match param_type {
                    "integer" => {
                        let info = get_int_param_safe(cam_id, param_name)?;
                        ParamValue::Int {
                            val: info.current,
                            range: Range {
                                min: info.min,
                                max: info.max,
                            },
                            inc: info.inc,
                        }
                    }
                    "float" => {
                        let info = get_float_param_safe(cam_id, param_name)?;
                        ParamValue::Float {
                            val: info.current,
                            range: Range {
                                min: info.min,
                                max: info.max,
                            },
                        }
                    }
                    "enum" => {
                        let info = get_enum_param_safe(cam_id, param_name)?;
                        ParamValue::Enum {
                            val: info.current,
                            supported: info.supported,
                        }
                    }
                    "bool" => {
                        let val = get_bool_param_safe(cam_id, param_name)?;
                        ParamValue::Bool { val }
                    }
                    "string" => {
                        let info = get_string_param_safe(cam_id, param_name)?;

                        ParamValue::String {
                            val: info.current,
                            max_length: info.max_length,
                        }
                    }
                    other => anyhow::bail!("Unsupported parameter type: {}", other),
                },
            );
        }
    }
    Ok(param_table)
}

#[derive(Debug, Serialize, Deserialize)]
struct Parameter {
    key: String,
    #[serde(rename = "ref")]
    param_ref: String,
    #[serde(rename = "type")]
    param_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Group {
    group_name: String,
    parameters: Vec<Parameter>,
}
fn init_sdk() -> Result<()> {
    let ret;
    unsafe {
        ret = init();
    }
    ensure!(
        ret.code == 0,
        "[cam err03] 相机SDK初始化失败；{}",
        stringify_err(ret)
    );
    Ok(())
}

fn init_cameras(camera_number: u32) -> Result<(), anyhow::Error> {
    let mut res: u32 = 0;
    let ret;
    unsafe {
        ret = enumerate_devices(&mut res as *mut u32);
        set_float_param(0, CString::new("ExposureTime").unwrap().as_ptr(), 10000.0);
    }
    ensure!(ret.code == 0, "枚举设备失败；{}", stringify_err(ret));
    ensure!(res != 0, "未发现设备");
    ensure!(res == camera_number, "发现设备数量与配置不符");

    info!("发现相机数量: {}", res);
    Ok(())
}

// fn config_camera(cam_id: u32) -> Result<(), anyhow::Error> {
//     let config::Camera {
//         exposure_time,
//         exposure_auto,
//         gain,
//         gain_auto,
//         width,
//         height,
//         ..
//     } = CONFIG.get().camera.lock().expect("锁中毒");
//     let mut ret;
//     #[cfg(feature = "hikvision")]
//     {
//         unsafe {
//             // ExposureAuto(自动曝光) 枚举值：0: 自动曝光关闭，1: 自动曝光一次 2: 自动曝光连续。参见海康相机用户手册
//             ret = set_enum_param(
//                 cam_id,
//                 CString::new("ExposureAuto").unwrap().as_ptr(),
//                 if exposure_auto { 2 } else { 0 },
//             );
//         }
//         ensure!(ret.code == 0, "设置曝光模式失败；{}", stringify_err(ret));
//         if !exposure_auto {
//             unsafe {
//                 // ExposureTime(曝光时间) 单位：微秒
//                 ret = set_float_param(
//                     cam_id,
//                     CString::new("ExposureTime").unwrap().as_ptr(),
//                     exposure_time,
//                 );
//             }
//             ensure!(ret.code == 0, "设置曝光时间失败；{}", stringify_err(ret));
//         }

//         unsafe {
//             // GainAuto(自动增益) 枚举值：0: 自动增益关闭，1: 自动增益一次 2: 自动增益连续。参见海康相机用户手册
//             ret = set_enum_param(
//                 cam_id,
//                 CString::new("GainAuto").unwrap().as_ptr(),
//                 if gain_auto { 2 } else { 0 },
//             );
//         }
//         ensure!(ret.code == 0, "设置增益模式失败；{}", stringify_err(ret));

//         if !gain_auto {
//             unsafe {
//                 // Gain(增益) 单位：分贝
//                 ret = set_float_param(cam_id, CString::new("Gain").unwrap().as_ptr(), gain);
//             }
//             ensure!(ret.code == 0, "设置增益失败；{}", stringify_err(ret));
//         }

//         unsafe {
//             // Width(图像宽度) 单位：像素
//             ret = set_int_param(cam_id, CString::new("Width").unwrap().as_ptr(), width);
//         }
//         ensure!(ret.code == 0, "设置图像宽度失败；{}", stringify_err(ret));

//         unsafe {
//             // Height(图像高度) 单位：像素
//             ret = set_int_param(cam_id, CString::new("Height").unwrap().as_ptr(), height);
//         }
//         ensure!(ret.code == 0, "设置图像高度失败；{}", stringify_err(ret));
//     }
//     Ok(())
// }

pub fn test() {
    init_sdk().unwrap();
    init_cameras(1).unwrap();
    let s = std::fs::read_to_string("cfg/hikvision/definition.toml").unwrap();
    let groups: Vec<Group> = toml::from_str(&s).unwrap();
    let res = query_parameters(0, &groups).unwrap();
    println!("{:?}", res);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        init_sdk().unwrap();
        init_cameras(1).unwrap();
        let s = std::fs::read_to_string("cfg/hikvision/definition.toml").unwrap();
        let groups: Vec<Group> = toml::from_str(&s).unwrap();
        let res = query_parameters(0, &groups).unwrap();
        println!("{:?}", res);
    }
}
