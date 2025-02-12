use anyhow::anyhow;

use crossbeam_channel::{unbounded as channel, Receiver, Sender, TryRecvError};

use std::{
    collections::VecDeque,
    sync::atomic::{AtomicBool, Ordering},
};

pub use log::error;

static STOP_SIG: AtomicBool = AtomicBool::new(false);

/// 检查是否已发出停止信号
///
/// # 返回值
/// 如果已发出停止信号，返回 `true`，否则返回 `false`
pub fn is_stopped() -> bool {
    STOP_SIG.load(Ordering::Relaxed)
}

/// 发出停止信号，通知所有线程停止工作
pub fn stop_all() {
    STOP_SIG.store(true, Ordering::Relaxed);
}

/// 确保条件为真，否则记录错误日志并发出停止信号
///
/// # 参数
/// - `$cond`: 要检查的条件表达式
/// - `$log`: 如果条件为假时记录的错误日志信息
#[macro_export]
macro_rules! ensure_or_stop {
    ($cond:expr, $log:expr) => {
        if !$cond {
            $crate::error!("{}", $log);
            $crate::stop_all();
            return;
        }
    };
}

#[macro_export]
macro_rules! unwrap_or_stop {
    ($wrap_value:expr) => {
        match $wrap_value {
            Ok(val) => val,
            Err(e) => {
                $crate::error!("{}", e);
                $crate::stop_all();
                return;
            }
        }
    };
}

#[macro_export]
macro_rules! expect_or_stop {
    ($wrap_value:expr, $log:expr) => {
        match $wrap_value {
            Ok(val) => val,
            Err(e) => {
                $crate::error!("{}", e);
                $crate::error!("{}", $log);
                $crate::stop_all();
                return;
            }
        }
    };
}

/// 管道的发送端结构
/// 管理数据的发送缓冲区和发送逻辑
pub struct TubeSend<T> {
    /// 内部缓冲队列，存储待发送的数据
    dishes: VecDeque<Box<T>>,
    /// 用于发送数据的通道
    supply: Sender<Box<T>>,
    /// 用于回收缓冲区的通道
    recycle: Receiver<Box<T>>,
}

/// 管道的接收端结构
/// 负责接收数据并支持回收空闲缓冲区
pub struct TubeRecv<T> {
    /// 用于接收数据的通道
    fetch: Receiver<Box<T>>,
    /// 用于回收缓冲区的发送端
    refund: Sender<Box<T>>,
}

/// 创建一个管道（Tube），包括发送端和接收端
///
/// # 泛型参数
/// - `T`: 数据类型，需要实现 `Default` trait，用于初始化缓冲区
///
/// # 返回值
/// 返回一个 `(TubeSend<T>, TubeRecv<T>)` 元组，分别表示发送端和接收端
pub fn new_tube<T>() -> (TubeSend<T>, TubeRecv<T>)
where
    T: Default,
{
    // 创建用于数据传输的通道
    let (supply, fetch) = channel();
    // 创建用于缓冲区回收的通道
    let (refund, recycle) = channel();

    // 初始化缓冲区，包含两个默认的数据项
    let mut dishes = VecDeque::new();
    dishes.push_back(Box::new(Default::default()));
    dishes.push_back(Box::new(Default::default()));

    // 构造发送端和接收端
    let tube_send = TubeSend {
        dishes,
        supply,
        recycle,
    };
    let tube_recv = TubeRecv { fetch, refund };

    (tube_send, tube_recv)
}

impl<T> TubeSend<T> {
    /// 获取发送缓冲区的第一个元素的可变引用
    ///
    /// # 返回值
    /// 返回缓冲区队列中的第一个元素。
    /// 如果队列为空，将触发 panic。
    pub fn get_send_buffer(&mut self) -> &mut Box<T> {
        self.dishes.front_mut().expect("[Tube] 数据结构逻辑错误")
    }

    /// 将缓冲区中的数据发送到接收端
    ///
    /// # 返回值
    /// - `Ok(())` 表示成功发送。
    /// - `Err(anyhow::Error)` 表示管道已关闭。
    pub fn send(&mut self) -> Result<(), anyhow::Error> {
        // 如果缓冲区只剩一个数据，尝试从回收通道中获取空缓冲区
        // 保证发送端至少有一个缓冲区可用
        if self.dishes.len() == 1 {
            match self.recycle.try_recv() {
                Ok(empty_dish) => self.dishes.push_back(empty_dish),
                Err(TryRecvError::Empty) => return Ok(()), // 无可用缓冲区
                Err(TryRecvError::Disconnected) => {
                    return Err(anyhow!("[utility] 管道已关闭，发送被阻止"))
                }
            }
        }
        // 发送缓冲区中的第一个数据
        self.supply
            .send(self.dishes.pop_front().expect("[Tube] 数据结构逻辑错误"))
            .map_err(|_| anyhow!("[utility] 管道已关闭，发送被阻止"))
    }
}

impl<T> TubeRecv<T> {
    /// 从接收通道中获取数据
    ///
    /// # 返回值
    /// - `Ok(Box<T>)` 表示成功接收到数据。
    /// - `Err(anyhow::Error)` 表示发送端已关闭。
    pub fn recv(&self) -> Result<Box<T>, anyhow::Error> {
        self.fetch
            .recv()
            .map_err(|_| anyhow!("[utility] 管道已关闭，接收被阻止"))
    }

    /// 回收空缓冲区，将其返还给发送端
    ///
    /// # 参数
    /// - `empty_dish`: 要回收的空缓冲区。
    ///
    /// # 返回值
    /// - `Ok(())` 表示成功回收。
    /// - `Err(anyhow::Error)` 表示发送端已关闭。
    pub fn recycle(&self, empty_dish: Box<T>) -> Result<(), anyhow::Error> {
        self.refund
            .send(empty_dish)
            .map_err(|_| anyhow!("[utility] 管道已关闭，回收被阻止"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug, PartialEq)]
    struct TestData {
        value: i32,
        payload: Vec<u8>,
    }

    #[test]
    fn test_tube_send_recv() {
        let (mut tube_send, tube_recv) = new_tube::<TestData>();

        // 获取发送缓冲区并修改数据
        let send_buffer = tube_send.get_send_buffer();
        send_buffer.value = 42;
        send_buffer.payload = vec![1, 2, 3];

        // 发送
        assert!(tube_send.send().is_ok());

        // 接收
        let recv_buffer = tube_recv.recv().unwrap();
        assert_eq!(recv_buffer.value, 42);
    }

    #[test]
    fn test_tube_recycle() {
        let (mut tube_send, tube_recv) = new_tube::<TestData>();

        // 获取发送缓冲区并修改数据
        let send_buffer = tube_send.get_send_buffer();
        send_buffer.value = 42;
        send_buffer.payload = vec![1, 2, 3];

        // 发送
        assert!(tube_send.send().is_ok());

        // 接收
        let recv_buffer = tube_recv.recv().unwrap();
        assert_eq!(recv_buffer.value, 42);

        // 回收接收到的缓冲区
        assert!(tube_recv.recycle(recv_buffer).is_ok());

        // 再次获取发送缓冲区
        let send_buffer = tube_send.get_send_buffer();
        assert_eq!(send_buffer.value, 0); // 0是默认值
    }

    #[test]
    fn test_tube_shutdown() {
        let (mut tube_send, tube_recv) = new_tube::<TestData>();

        // 模拟管道关闭
        drop(tube_recv);

        // 尝试发送数据
        assert!(matches!(
            tube_send.send(),
            Err(e) if e.to_string() == "[utility] 管道已关闭，发送被阻止"
        ));
    }

    #[test]
    fn test_tube_recv_shutdown() {
        let (tube_send, tube_recv) = new_tube::<TestData>();

        // 模拟管道关闭
        drop(tube_send);

        // 尝试接收数据
        assert!(matches!(
            tube_recv.recv(),
            Err(e) if e.to_string() == "[utility] 管道已关闭，接收被阻止"
        ));
    }

    #[test]
    fn test_tube_recycle_shutdown() {
        let (mut tube_send, tube_recv) = new_tube::<TestData>();

        assert!(tube_send.send().is_ok());

        let recv_buffer = tube_recv.recv();
        assert!(recv_buffer.is_ok());

        // 模拟管道关闭
        drop(tube_send);

        // 尝试回收缓冲区
        let empty_dish = recv_buffer.unwrap();
        assert!(matches!(
            tube_recv.recycle(empty_dish),
            Err(e) if e.to_string() == "[utility] 管道已关闭，回收被阻止"
        ));
    }
}
