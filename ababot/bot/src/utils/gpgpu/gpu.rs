use std::{any::TypeId, error::Error, result};

use gpgpu::{BufOps, DescriptorSet, Framework, GpuBuffer, GpuBufferUsage, Kernel, Program, Shader};

use super::worker::{GpuWorkType, Vec3, Worker};

pub async fn gpu_task<T>(
    mut receiver: tokio::sync::mpsc::Receiver<Worker<T>>,
) -> Result<Vec<T>, Box<dyn Error>>
where
    T: GpuWorkType,
{
    let fw = Framework::default();

    loop {
        let worker = match receiver.recv().await {
            Some(w) => w,
            None => continue,
        };

        let shader = Shader::from_wgsl_file(&fw, &worker.file_name)?;

        let res = execute(&fw, shader, worker)?;
        return Ok(res);
    }
}

pub async fn gpu_task_2<T>(worker: Worker<T>) -> Result<Vec<T>, Box<dyn Error>>
where
    T: GpuWorkType,
{
    let fw = Framework::default();

    // loop {
    //     let worker = match receiver.recv().await {
    //         Some(w) => w,
    //         None => continue,
    //     };

    let shader = Shader::from_wgsl_file(&fw, &worker.file_name)?;

    execute(&fw, shader, worker)
    // }
}

fn execute<T>(fw: &Framework, shader: Shader, worker: Worker<T>) -> Result<Vec<T>, Box<dyn Error>>
where
    T: GpuWorkType,
{
    let mut gpu_in_buffs: Vec<GpuBuffer<T>> = Vec::with_capacity(worker.work_data.len() + 1);

    for work in worker.work_data {
        let noe = GpuBuffer::from_slice(&fw, &work);
        gpu_in_buffs.push(noe);
    }

    let gpu_out_buff = GpuBuffer::<T>::with_capacity(&fw, worker.out_data.len() as u64);

    let mut descriptor_set = DescriptorSet::default();
        // .bind_buffer(&gpu_in_buffs[0], GpuBufferUsage::ReadOnly)
        // .bind_buffer(&gpu_out_buff, GpuBufferUsage::ReadWrite);

    for buff in gpu_in_buffs.iter() {
        descriptor_set = descriptor_set.bind_buffer(buff, GpuBufferUsage::ReadOnly);
    }
    descriptor_set = descriptor_set.bind_buffer(&gpu_out_buff, GpuBufferUsage::ReadWrite);

    let kernel = Program::new(&shader, "main").add_descriptor_set(descriptor_set);

    let threads = worker.work_size;

    Kernel::new(&fw, kernel).enqueue(threads.x as u32, threads.y as u32, threads.z as u32);

    let output = gpu_out_buff.read_vec_blocking()?;
    Ok(output)
}

pub async fn test_gpgpu() -> Result<(), Box<dyn Error>> {
    let file_path = String::from("bot/src/utils/gpgpu/compute.wgsl");

    let cpu_data = (0..100).into_iter().collect::<Vec<u32>>();
    let result: Vec<u32> = (0..100).into_iter().map(|_x: u32| 0).collect();

    println!("len of vec: {}", result.len());

    let thread_group = Vec3::default();

    let worker = Worker {
        file_name: file_path,
        work_data: vec![cpu_data],
        out_data: result,
        work_size: thread_group,
    };

    let cpu_computed_data = (0..10000).into_iter().map(|x| x * 2).collect::<Vec<u32>>();

    // sender.send(worker).await?;

    let res = gpu_task_2(worker).await?;

    for (a, b) in cpu_computed_data.into_iter().zip(res) {
        // println!("{} {}", a, b);
        assert_eq!(a, b);
    }
    println!("Test passed");

    Ok(())
}
