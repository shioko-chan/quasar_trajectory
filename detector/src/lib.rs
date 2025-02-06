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

pub fn detector(terminate: Arc<AtomicBool>) -> JoinHandle<Result<(), DetectorError>> {
    thread::spawn(move || {
        let mut res: u32 = 0;
        let mut ret;
        unsafe {
            ret = init();
        }
        if ret.code != 0 {
            return Err(DetectorError::CameraError(format!(
                "相机初始化失败: {:?}",
                ret
            )));
        }
        unsafe {
            ret = enumerate_devices(&mut res as *mut u32);
        }
        if ret.code != 0 {
            return Err(DetectorError::CameraError(format!(
                "枚举设备失败: {:?}",
                ret
            )));
        }
        if res == 0 {
            return Err(DetectorError::CameraError("No device found".to_string()));
        }
        info!("检测器已启动, 相机数量: {}", res);
        const cam_id: u32 = 0;
        let exposure_time = CONFIG
            .lock()
            .expect("锁中毒")
            .as_ref()
            .expect("CONFIG未初始化")
            .camera
            .exposure_time;

        unsafe {
            ret = set_enum_param(cam_id, CString::new("ExposureAuto").unwrap().as_ptr(), 0);
        }
        if ret.code != 0 {
            return Err(DetectorError::CameraError(format!(
                "设置曝光模式失败: {:?}",
                ret
            )));
        }
        unsafe {
            ret = set_float_param(
                cam_id,
                CString::new("ExposureTime").unwrap().as_ptr(),
                exposure_time,
            );
        }
        if ret.code != 0 {
            return Err(DetectorError::CameraError(format!(
                "设置曝光时间失败: {:?}",
                ret
            )));
        }
        #[cfg(feature = "visualize")]
        {
            let mut cnt = 0;
            let mut start = std::time::Instant::now();
            let mut img = vec![0u8; 1440 * 1080 * 3];
            while !terminate.load(atomic::Ordering::Relaxed) {
                unsafe {
                    ret = get_frame(cam_id, img.as_mut_ptr());
                }
                if ret.code != 0 {
                    return Err(DetectorError::CameraError(format!(
                        "获取图像失败: {:?}",
                        ret
                    )));
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
