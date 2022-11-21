use std::error::Error;

use gpgpu::{BufOps, DescriptorSet, Framework, GpuBuffer, GpuBufferUsage, Kernel, Program, Shader};
use tokio::sync::mpsc;

use super::{
    channels::{GpuTask, GPU},
    worker::{GpuWork, GpuWorkType},
};

pub async fn gpu_handler(receiver: &mut mpsc::Receiver<GPU>) -> Result<(), Box<dyn Error>> {
    let mut fw = Framework::default();
    loop {
        let data = match receiver.recv().await {
            Some(data) => data,
            None => continue,
        };

        match data {
            GPU::GpuU8(data) => gpu_task(data, &mut fw)?,
            GPU::GpuU16(data) => gpu_task(data, &mut fw)?,
            GPU::GpuU32(data) => gpu_task(data, &mut fw)?,
            GPU::GpuU64(data) => gpu_task(data, &mut fw)?,
            GPU::GpuI8(data) => gpu_task(data, &mut fw)?,
            GPU::GpuI16(data) => gpu_task(data, &mut fw)?,
            GPU::GpuI32(data) => gpu_task(data, &mut fw)?,
            GPU::GpuI64(data) => gpu_task(data, &mut fw)?,
            GPU::GpuF32(data) => gpu_task(data, &mut fw)?,
            GPU::GpuF64(data) => gpu_task(data, &mut fw)?,
        }
    }
}

fn gpu_task<T>(task: GpuTask<T>, fw: &mut Framework) -> Result<(), Box<dyn Error>>
where
    T: GpuWorkType,
{
    let sender = task.return_channel;
    let worker = task.data;

    let shader =  Shader::from_wgsl_file(fw, &worker.file_name).map_err(|e| {
        tracing::error!("Failed to load shader: {}", e);
        e
    })?;

    let res = execute(fw, shader, &worker)?;

    if sender.send(res).is_err() {
        println!("Failed to send data back to main thread");
        tracing::error!("Failed to send data back to main thread");
    }
    Ok(())
}

fn execute<T>(fw: &Framework, shader: Shader, worker: &GpuWork<T>) -> Result<Vec<T>, Box<dyn Error>>
where
    T: GpuWorkType,
{
    let mut gpu_in_buffs: Vec<GpuBuffer<T>> = Vec::with_capacity(worker.work_data.len() + 1);

    for work in worker.work_data.iter() {
        let noe = GpuBuffer::from_slice(fw, work);
        gpu_in_buffs.push(noe);
    }

    // Build descriptor set base
    let mut descriptor_set = DescriptorSet::default();

    // Bind all input buffers
    for buff in gpu_in_buffs.iter() {
        descriptor_set = descriptor_set.bind_buffer(buff, GpuBufferUsage::ReadOnly);
    }
    // Creating and binding output buffer
    let gpu_out_buff = GpuBuffer::<T>::with_capacity(fw, worker.out_data_len);
    descriptor_set = descriptor_set.bind_buffer(&gpu_out_buff, GpuBufferUsage::ReadWrite);

    let kernel = Program::new(&shader, "main").add_descriptor_set(descriptor_set);

    // Thread group
    let t = worker.work_size;

    // Execute kernel
    Kernel::new(fw, kernel).enqueue(t.x as u32, t.y as u32, t.z as u32);

    let output = gpu_out_buff.read_vec_blocking()?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::{error::Error, sync::Arc};

    use tokio::sync::mpsc;

    use crate::utils::gpgpu::{
        channels::{GpuTask, GPU},
        worker::{GpuWork, Vec3},
    };

    use super::gpu_handler;

    #[tokio::test]
    pub async fn test_gpgpu() -> Result<(), Box<dyn Error>> {
        let file_path = String::from("gpu/gpgpu_tests/compute.wgsl");

        let cpu_data = (0..100).into_iter().collect::<Vec<u32>>();

        let thread_group = Vec3::default();

        let worker = GpuWork {
            file_name: file_path,
            work_data: vec![cpu_data],
            out_data_len: 100_u64,
            work_size: thread_group,
        };
        let (left_mpsc, mut right_mpsc) = mpsc::channel::<GPU>(1);

        let (work, right) = GpuTask::new(worker);

        let cpu_computed_data = (0..10000).into_iter().map(|x| x * 2).collect::<Vec<u32>>();

        tokio::spawn(async move {
            if gpu_handler(&mut right_mpsc).await.is_err() {
                panic!("Failed to execute gpu task");
            }
        });

        left_mpsc.send(GPU::GpuU32(work)).await.unwrap();

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

        let thread_group = Vec3::default();

        let worker = GpuWork {
            file_name: file_path,
            work_data: vec![cpu_data],
            out_data_len: 25,
            work_size: thread_group,
        };
        let (work, right) = GpuTask::new(worker);

        let (left_mpsc, mut right_mpsc) = mpsc::channel::<GPU>(1);
        let cpu_computed_data = vec![
            0, 1, 7, 2, 5, 8, 16, 3, 19, 6, 14, 9, 9, 17, 17, 4, 12, 20, 20, 7, 7, 15, 15, 10, 23,
        ];

        tokio::spawn(async move {
            if gpu_handler(&mut right_mpsc).await.is_err() {
                panic!("Failed to execute gpu task");
            }
        });

        left_mpsc.send(GPU::GpuU32(work)).await.unwrap();

        let res = right.await.unwrap();

        for (a, b) in cpu_computed_data.into_iter().zip(res) {
            assert_eq!(a, b);
        }
    }

    #[tokio::test]
    async fn test_for_loop() {
        let file_path = String::from("gpu/gpgpu_tests/for_loop.wgsl");

        let cpu_data = (0..100).into_iter().collect::<Vec<u32>>();

        let thread_group = Vec3::default();

        let worker = GpuWork {
            file_name: file_path,
            work_data: vec![cpu_data],
            out_data_len: 100,
            work_size: thread_group,
        };
        let (work, right) = GpuTask::new(worker);

        let (left_mpsc, mut right_mpsc) = mpsc::channel::<GPU>(1);
        let base: u32 = 2;
        let cpu_computed_data = (0..100)
            .into_iter()
            .map(|x| x * base.pow(10))
            .collect::<Vec<u32>>();

        tokio::spawn(async move {
            if gpu_handler(&mut right_mpsc).await.is_err() {
                panic!("Failed to execute gpu task");
            }
        });

        left_mpsc.send(GPU::GpuU32(work)).await.unwrap();

        let res = right.await.unwrap();

        assert_eq!(res.len(), 100);
        for (a, b) in cpu_computed_data.into_iter().zip(res) {
            assert_eq!(a, b);
        }
    }

    #[tokio::test]
    async fn test_multiple_tasks() {
        let file_path = String::from("gpu/gpgpu_tests/for_loop.wgsl");
        let file_path_2 = String::from("gpu/gpgpu_tests/compute.wgsl");

        let cpu_data = (0..100).into_iter().collect::<Vec<u32>>();
        let cpu_data_2 = (0..100).into_iter().collect::<Vec<u32>>();

        let thread_group = Vec3::default();

        let worker = GpuWork {
            file_name: file_path,
            work_data: vec![cpu_data],
            out_data_len: 100,
            work_size: thread_group,
        };
        let worker_2 = GpuWork {
            file_name: file_path_2,
            work_data: vec![cpu_data_2],
            out_data_len: 100,
            work_size: thread_group,
        };
        let (work, right_1) = GpuTask::new(worker);
        let (work_2, right_2) = GpuTask::new(worker_2);

        let (left_mpsc, mut right_mpsc) = mpsc::channel::<GPU>(1);
        let base: u32 = 2;
        let cpu_computed_data = (0..100)
            .into_iter()
            .map(|x| x * base.pow(10))
            .collect::<Vec<u32>>();
        let cpu_computed_data_2 = (0..10000).into_iter().map(|x| x * 2).collect::<Vec<u32>>();

        tokio::spawn(async move {
            if gpu_handler(&mut right_mpsc).await.is_err() {
                panic!("Failed to execute gpu task");
            }
        });
        let left_arc = Arc::new(left_mpsc);
        let left_arc_2 = left_arc.clone();
        let left_arc_3 = left_arc;
        tokio::spawn(async move {
            left_arc_2.send(GPU::GpuU32(work)).await.unwrap();

            let res = right_1.await.unwrap();

            for (a, b) in cpu_computed_data.into_iter().zip(res) {
                assert_eq!(a, b);
            }
        });

        tokio::spawn(async move {
            left_arc_3.send(GPU::GpuU32(work_2)).await.unwrap();

            let res = right_2.await.unwrap();

            for (a, b) in cpu_computed_data_2.into_iter().zip(res) {
                assert_eq!(a, b);
            }
        });
    }
}
