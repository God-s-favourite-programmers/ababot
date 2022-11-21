use std::sync::Arc;

use serenity::prelude::TypeMapKey;
use tokio::sync::{mpsc::Sender, oneshot};

use super::worker::{GpuWork, GpuWorkType};

#[derive(Debug)]
pub struct GpuTask<T>
where
    T: GpuWorkType,
{
    pub data: GpuWork<T>,
    pub return_channel: oneshot::Sender<Vec<T>>,
}

impl<T> GpuTask<T>
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
#[derive(Debug)]
pub enum GPU {
    GpuU8(GpuTask<u8>),
    GpuU16(GpuTask<u16>),
    GpuU32(GpuTask<u32>),
    GpuU64(GpuTask<u64>),
    GpuI8(GpuTask<i8>),
    GpuI16(GpuTask<i16>),
    GpuI32(GpuTask<i32>),
    GpuI64(GpuTask<i64>),
    GpuF32(GpuTask<f32>),
    GpuF64(GpuTask<f64>),
}

impl TypeMapKey for GPU {
    type Value = Arc<Sender<GPU>>;
}
