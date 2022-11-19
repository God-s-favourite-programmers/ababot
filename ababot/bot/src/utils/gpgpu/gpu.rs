use std::error::Error;

use gpgpu::{BufOps, DescriptorSet, Framework, GpuBuffer, GpuBufferUsage, Kernel, Program, Shader};
use tokio::sync::mpsc;

use super::{
    channels::GpuTaskChannel,
    worker::{GpuWork, GpuWorkType},
};

pub async fn gpu_task<T>(
    receiver: &mut mpsc::Receiver<GpuTaskChannel<T>>,
) -> Result<(), Box<dyn Error>>
where
    T: GpuWorkType,
{
    let fw = Framework::default();
    loop {
        let task = match receiver.recv().await {
            Some(w) => w,
            None => continue,
        };

        let sender = task.return_channel;
        let worker = task.data;

        let shader = Shader::from_wgsl_file(&fw, &worker.file_name)?;

        let res = execute(&fw, shader, &worker)?;

        if let Err(_) = sender.send(res) {
            println!("Failed to send data back to main thread");
        };
    }
}

fn execute<T>(fw: &Framework, shader: Shader, worker: &GpuWork<T>) -> Result<Vec<T>, Box<dyn Error>>
where
    T: GpuWorkType,
{
    let mut gpu_in_buffs: Vec<GpuBuffer<T>> = Vec::with_capacity(worker.work_data.len() + 1);

    for work in worker.work_data.iter() {
        let noe = GpuBuffer::from_slice(&fw, work);
        gpu_in_buffs.push(noe);
    }

    // Build descriptor set base
    let mut descriptor_set = DescriptorSet::default();

    // Bind all input buffers
    for buff in gpu_in_buffs.iter() {
        descriptor_set = descriptor_set.bind_buffer(buff, GpuBufferUsage::ReadOnly);
    }
    // Creating and binding output buffer
    let gpu_out_buff = GpuBuffer::<T>::with_capacity(&fw, worker.out_data.len() as u64);
    descriptor_set = descriptor_set.bind_buffer(&gpu_out_buff, GpuBufferUsage::ReadWrite);

    let kernel = Program::new(&shader, "main").add_descriptor_set(descriptor_set);

    // Thread group
    let t = worker.work_size;

    // Execute kernel
    Kernel::new(&fw, kernel).enqueue(t.x as u32, t.y as u32, t.z as u32);

    let output = gpu_out_buff.read_vec_blocking()?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio::sync::mpsc;

    use crate::utils::gpgpu::{
        channels::GpuTaskChannel,
        gpu::gpu_task,
        worker::{GpuWork, Vec3},
    };

    #[tokio::test]
    pub async fn test_gpgpu() -> Result<(), Box<dyn Error>> {
        let file_path = String::from("gpu/gpgpu_tests/compute.wgsl");

        let cpu_data = (0..100).into_iter().collect::<Vec<u32>>();
        let result: Vec<u32> = (0..100).into_iter().map(|_x: u32| 0).collect();

        let thread_group = Vec3::default();

        let worker = GpuWork {
            file_name: file_path,
            work_data: vec![cpu_data],
            out_data: result,
            work_size: thread_group,
        };
        let (left_mpsc, mut right_mpsc) = mpsc::channel::<GpuTaskChannel<u32>>(1);

        let (work, right) = GpuTaskChannel::new(worker);

        let cpu_computed_data = (0..10000).into_iter().map(|x| x * 2).collect::<Vec<u32>>();

        tokio::spawn(async move {
            if let Err(_) = gpu_task(&mut right_mpsc).await {
                panic!("Failed to execute gpu task");
            }
        });

        left_mpsc.send(work).await.unwrap();

        let res = right.await.unwrap();

        for (a, b) in cpu_computed_data.into_iter().zip(res) {
            assert_eq!(a, b);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_bpu_collatz() {
        let file_path = String::from("gpu/gpgpu_tests/collatz.wgsl");

        let cpu_data = (1..26).into_iter().collect::<Vec<u32>>();
        let result: Vec<u32> = vec![0; 25];

        let thread_group = Vec3::default();

        let worker = GpuWork {
            file_name: file_path,
            work_data: vec![cpu_data],
            out_data: result,
            work_size: thread_group,
        };
        let (work, right) = GpuTaskChannel::new(worker);

        let (left_mpsc, mut right_mpsc) = mpsc::channel::<GpuTaskChannel<u32>>(1);
        let cpu_computed_data = vec![
            0, 1, 7, 2, 5, 8, 16, 3, 19, 6, 14, 9, 9, 17, 17, 4, 12, 20, 20, 7, 7, 15, 15, 10, 23,
        ];

        tokio::spawn(async move {
            if let Err(_) = gpu_task(&mut right_mpsc).await {
                panic!("Failed to execute gpu task");
            }
        });

        left_mpsc.send(work).await.unwrap();

        let res = right.await.unwrap();

        for (a, b) in cpu_computed_data.into_iter().zip(res) {
            assert_eq!(a, b);
        }
    }
}
