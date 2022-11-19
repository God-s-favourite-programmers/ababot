use tokio::sync::oneshot;

use super::worker::{GpuWork, GpuWorkType};

#[derive(Debug)]
pub struct GpuTaskChannel<T>
where
    T: GpuWorkType,
{
    pub data: GpuWork<T>,
    pub return_channel: oneshot::Sender<Vec<T>>,
}

impl<T> GpuTaskChannel<T>
where
    T: GpuWorkType,
{
    pub fn new(data: GpuWork<T>) -> (Self, oneshot::Receiver<Vec<T>>) {
        let (return_channel, right) = oneshot::channel::<Vec<T>>();
        (
            Self {
                data,
                return_channel,
            },
            right,
        )
    }
}
