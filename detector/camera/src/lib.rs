#[cfg(feature = "hikvision")]
include!(concat!(env!("OUT_DIR"), "hikcamera/camera.rs"));

#[cfg(feature = "mindvision")]
include!(concat!(env!("OUT_DIR"), "mindcamera/camera.rs"));

use anyhow::{ensure, Result};
use log::{error, info, warn};
use utility::ensure_or_stop;

struct CameraThreadsHandle(Vec<JoinHandle<()>>);

impl CameraThreadsHandle {
    fn new(stop_signal: Arc<AtomicBool>, camera_number: u32) -> Result<Self> {
        init_sdk()?;
        init_cameras(camera_number)?;
        Ok((0..camera_number)
            .map(|cam_id| {
                let stop_signal = stop_signal.clone();
                thread::spawn(move || {
                    ensure_or_stop!(
                        stop_signal,
                        config_camera(cam_id).is_ok(),
                        "[cam err01] 相机配置失败"
                    );
                })
            })
            .collect())
    }
}

impl Drop for CameraThreadsHandle {
    fn drop(&mut self) {
        for handle in self.0.drain(..) {
            if let Err(e) = handle.join() {
                warn!("[cam err02] 相机线程退出失败: {:?}", e);
            }
        }
    }
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
    }
    ensure!(ret.code == 0, "枚举设备失败；{}", stringify_err(ret));
    ensure!(res != 0, "未发现设备");
    ensure!(res == camera_number, "发现设备数量与配置不符");

    info!("发现相机数量: {}", res);
    Ok(())
}

fn config_camera(cam_id: u32) -> Result<(), anyhow::Error> {
    let config::Camera {
        exposure_time,
        exposure_auto,
        gain,
        gain_auto,
        width,
        height,
        ..
    } = *CONFIG.get().camera.lock().expect("锁中毒");
    let mut ret;
    #[cfg(feature = "hikvision")]
    {
        unsafe {
            // ExposureAuto(自动曝光) 枚举值：0: 自动曝光关闭，1: 自动曝光一次 2: 自动曝光连续。参见海康相机用户手册
            ret = set_enum_param(
                cam_id,
                CString::new("ExposureAuto").unwrap().as_ptr(),
                if exposure_auto { 2 } else { 0 },
            );
        }
        ensure!(ret.code == 0, "设置曝光模式失败；{}", stringify_err(ret));
        if !exposure_auto {
            unsafe {
                // ExposureTime(曝光时间) 单位：微秒
                ret = set_float_param(
                    cam_id,
                    CString::new("ExposureTime").unwrap().as_ptr(),
                    exposure_time,
                );
            }
            ensure!(ret.code == 0, "设置曝光时间失败；{}", stringify_err(ret));
        }

        unsafe {
            // GainAuto(自动增益) 枚举值：0: 自动增益关闭，1: 自动增益一次 2: 自动增益连续。参见海康相机用户手册
            ret = set_enum_param(
                cam_id,
                CString::new("GainAuto").unwrap().as_ptr(),
                if gain_auto { 2 } else { 0 },
            );
        }
        ensure!(ret.code == 0, "设置增益模式失败；{}", stringify_err(ret));

        if !gain_auto {
            unsafe {
                // Gain(增益) 单位：分贝
                ret = set_float_param(cam_id, CString::new("Gain").unwrap().as_ptr(), gain);
            }
            ensure!(ret.code == 0, "设置增益失败；{}", stringify_err(ret));
        }

        unsafe {
            // Width(图像宽度) 单位：像素
            ret = set_int_param(cam_id, CString::new("Width").unwrap().as_ptr(), width);
        }
        ensure!(ret.code == 0, "设置图像宽度失败；{}", stringify_err(ret));

        unsafe {
            // Height(图像高度) 单位：像素
            ret = set_int_param(cam_id, CString::new("Height").unwrap().as_ptr(), height);
        }
        ensure!(ret.code == 0, "设置图像高度失败；{}", stringify_err(ret));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
