use std::{error::Error, result};

use gpgpu::{Framework, Shader, GpuBuffer, DescriptorSet, Program, Kernel, BufOps};

use super::worker::{GpuWorkType, Worker};


pub async fn gpu_task<T>(mut receiver: tokio::sync::mpsc::Receiver<Worker<T>>) -> Result<Vec<T>, Box<dyn Error>>
where T: GpuWorkType
{
    let fw = Framework::default();

    loop {
        let worker = match receiver.recv().await {
            Some(w) => w,
            None => continue,
        };

        let shader = Shader::from_wgsl_file(&fw, &worker.file_name)?;

        let res = execute(&fw, shader, worker)?;
        return Ok(vec![]);
    }

}

macro_rules! bind_buffer_group {
    () => {
        
    };
}

fn execute<T>(fw: &Framework, shader: Shader, worker: Worker<T>) -> Result<Vec<T>, Box<dyn Error>>
where T: GpuWorkType
{
    let gpu_in_buffs: Vec<GpuBuffer<T>> = Vec::with_capacity(worker.work_data.len() + 1);

    for work in worker.work_data {
        let noe = GpuBuffer::from_slice(&fw, &work);
    }

    let descriptor_set = DescriptorSet::default();

    let kernel = Program::new(&shader, "main").add_descriptor_set(descriptor_set);

    let threads = worker.work_size;

    Kernel::new(&fw, kernel).enqueue(threads.x as u32, threads.y as u32, threads.z as u32);
    Ok(vec![])
}