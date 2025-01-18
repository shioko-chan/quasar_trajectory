#[cfg(not(feature = "use_crossbeam"))]
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

use std::collections::VecDeque;

#[cfg(feature = "use_crossbeam")]
use crossbeam_channel::{unbounded as channel, Receiver, Sender, TryRecvError};

use thiserror::Error;

/// 表示管道可能的错误类型
#[derive(Debug, Error, PartialEq)]
pub enum TubeError {
    /// 管道已经关闭，无法发送或接收数据
    #[error("This tube has shut down.")]
    TubeShutdown,
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

/// 当管道结构出现逻辑错误时的 panic 信息
const TUBE_PANIC: &str = "[Tube]: data structure collapsed";

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
        self.dishes.front_mut().expect(TUBE_PANIC)
    }

    /// 将缓冲区中的数据发送到接收端
    ///
    /// # 返回值
    /// - `Ok(())` 表示成功发送。
    /// - `Err(TubeError::TubeShutdown)` 表示管道已关闭。
    pub fn send(&mut self) -> Result<(), TubeError> {
        // 如果缓冲区只剩一个数据，尝试从回收通道中获取空缓冲区
        // 保证发送端至少有一个缓冲区可用
        if self.dishes.len() == 1 {
            match self.recycle.try_recv() {
                Ok(empty_dish) => self.dishes.push_back(empty_dish),
                Err(TryRecvError::Empty) => return Ok(()), // 无可用缓冲区
                Err(TryRecvError::Disconnected) => return Err(TubeError::TubeShutdown),
            }
        }
        // 发送缓冲区中的第一个数据
        self.supply
            .send(self.dishes.pop_front().expect(TUBE_PANIC))
            .map_err(|_| TubeError::TubeShutdown)
    }
}

impl<T> TubeRecv<T> {
    /// 从接收通道中获取数据
    ///
    /// # 返回值
    /// - `Ok(Box<T>)` 表示成功接收到数据。
    /// - `Err(TubeError::TubeShutdown)` 表示发送端已关闭。
    pub fn recv(&self) -> Result<Box<T>, TubeError> {
        self.fetch.recv().map_err(|_| TubeError::TubeShutdown)
    }

    /// 回收空缓冲区，将其返还给发送端
    ///
    /// # 参数
    /// - `empty_dish`: 要回收的空缓冲区。
    ///
    /// # 返回值
    /// - `Ok(())` 表示成功回收。
    /// - `Err(TubeError::TubeShutdown)` 表示发送端已关闭。
    pub fn recycle(&self, empty_dish: Box<T>) -> Result<(), TubeError> {
        self.refund
            .send(empty_dish)
            .map_err(|_| TubeError::TubeShutdown)
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
        assert_eq!(tube_send.send(), Err(TubeError::TubeShutdown));
    }

    #[test]
    fn test_tube_recv_shutdown() {
        let (tube_send, tube_recv) = new_tube::<TestData>();

        // 模拟管道关闭
        drop(tube_send);

        // 尝试接收数据
        assert_eq!(tube_recv.recv(), Err(TubeError::TubeShutdown));
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
        assert_eq!(tube_recv.recycle(empty_dish), Err(TubeError::TubeShutdown));
    }
}
