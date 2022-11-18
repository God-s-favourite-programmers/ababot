use tokio::sync::oneshot;

use super::worker::{GpuWorkType, Worker};

#[derive(Debug)]
pub struct GpuTaskChannel<T> where T: GpuWorkType {
    pub data: Worker<T>,
    pub return_channel: oneshot::Sender<Vec<T>>,
}

impl<T> GpuTaskChannel<T> where T: GpuWorkType {
    pub fn new(data: Worker<T>, return_channel: oneshot::Sender<Vec<T>>) -> Self {
        Self {
            data,
            return_channel,
        }
    }
}
