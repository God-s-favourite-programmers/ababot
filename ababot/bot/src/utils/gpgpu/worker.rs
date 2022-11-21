use bytemuck::Pod;

#[derive(Debug)]
pub struct GpuWork<T>
where
    T: GpuWorkType,
{
    pub file_name: String,
    pub work_data: Vec<Vec<T>>,
    pub out_data_len: u64,
    pub work_size: Vec3,
}

pub trait GpuWorkType
where
    Self: Pod,
{
}
impl GpuWorkType for u8 {}
impl GpuWorkType for u16 {}
impl GpuWorkType for u32 {}
impl GpuWorkType for u64 {}
impl GpuWorkType for i8 {}
impl GpuWorkType for i16 {}
impl GpuWorkType for i32 {}
impl GpuWorkType for i64 {}
impl GpuWorkType for f32 {}
impl GpuWorkType for f64 {}

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

impl Default for Vec3 {
    fn default() -> Self {
        Self {
            x: u16::MAX,
            y: 1,
            z: 1,
        }
    }
}
