#[cfg(feature = "hikvision")]
include!(concat!(env!("OUT_DIR"), "/hikcamera/camera.rs"));

#[cfg(feature = "mindvision")]
include!(concat!(env!("OUT_DIR"), "hikcamera/camera.rs"));

use config::CONFIG;
use log::info;
use std::ffi::CString;
use std::{
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
    thread::{self, JoinHandle},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DetectorError {
    #[error("Camera error occurred: {0}")]
    CameraError(String),
}

fn stringify_err(ret: api_error) -> String {
    format!(
        "错误来源：{}， 错误码：{:X}",
        if ret.is_hik_err == 1 {
            "`第三方相机SDK`"
        } else {
            "`相机API`"
        },
        ret.code as usize
    )
}

fn init_sdk() -> Result<(), DetectorError> {
    let ret;
    unsafe {
        ret = init();
    }
    if ret.code != 0 {
        return Err(DetectorError::CameraError(format!(
            "相机初始化失败；{}",
            stringify_err(ret)
        )));
    }
    Ok(())
}

fn init_cameras() -> Result<(), DetectorError> {
    let mut res: u32 = 0;
    let ret;
    unsafe {
        ret = enumerate_devices(&mut res as *mut u32);
    }
    if ret.code != 0 {
        return Err(DetectorError::CameraError(format!(
            "枚举设备失败: {}",
            stringify_err(ret)
        )));
    }
    if res == 0 {
        return Err(DetectorError::CameraError("未发现设备".to_string()));
    }
    info!("发现相机数量: {}", res);
    Ok(())
}

pub fn detector(terminate: Arc<AtomicBool>) -> JoinHandle<Result<(), DetectorError>> {
    thread::spawn(move || {
        init_sdk()?;
        init_cameras()?;

        const CAM_ID: u32 = 0;
        let exposure_time = CONFIG
            .lock()
            .expect("锁中毒")
            .as_ref()
            .expect("CONFIG未初始化")
            .camera
            .exposure_time;
        let mut ret;
        unsafe {
            ret = set_enum_param(CAM_ID, CString::new("ExposureAuto").unwrap().as_ptr(), 0);
        }
        if ret.code != 0 {
            return Err(DetectorError::CameraError(format!(
                "设置曝光模式失败: {}",
                stringify_err(ret)
            )));
        }
        unsafe {
            set_enum_param(CAM_ID, CString::new("ExposureMode").unwrap().as_ptr(), 0);
        }
        unsafe {
            ret = set_float_param(
                CAM_ID,
                CString::new("ExposureTime").unwrap().as_ptr(),
                exposure_time,
            );
        }
        if ret.code != 0 {
            return Err(DetectorError::CameraError(format!(
                "设置曝光时间失败: {}",
                stringify_err(ret)
            )));
        }
        #[cfg(feature = "visualize")]
        {
            let mut cnt = 0;
            let mut start = std::time::Instant::now();
            let mut img = vec![0u8; 1440 * 1080 * 3];
            while !terminate.load(atomic::Ordering::Relaxed) {
                unsafe {
                    ret = get_frame(CAM_ID);
                }

                // if ret.code != 0 {
                //     return Err(DetectorError::CameraError(format!(
                //         "获取图像失败: {}",
                //         stringify_err(ret)
                //     )));
                // }
                if ret.code != 0 {
                    continue;
                }
                cnt += 1;
                if cnt == 100 {
                    let elapsed = start.elapsed();
                    info!("fps: {}", 100.0 / elapsed.as_secs_f64());
                    start = std::time::Instant::now();
                    cnt = 0;
                }
            }
        }
        #[cfg(not(feature = "visualize"))]
        {
            while !terminate.load(atomic::Ordering::Relaxed) {
                unsafe {
                    // get_frame();
                }
            }
        }
        unsafe {
            final_();
        }
        info!("检测器已退出");
        Ok(())
    })
}
