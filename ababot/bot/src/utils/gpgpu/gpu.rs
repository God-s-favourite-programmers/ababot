use std::error::Error;

use gpgpu::{BufOps, DescriptorSet, Framework, GpuBuffer, GpuBufferUsage, Kernel, Program, Shader};

use super::worker::{GpuWorkType, Worker};

pub fn gpu_task<T>(
    receiver: bichannel::Channel<Worker<T>, Worker<T>>,
) -> Result<(), Box<dyn Error>>
where
    T: GpuWorkType,
{
    let fw = Framework::default();
    println!("Running gpu task");
    loop {
        let worker = match receiver.recv() {
            Ok(w) => w,
            Err(_e) => continue,
        };
        println!("Received work");

        let shader = Shader::from_wgsl_file(&fw, &worker.file_name)?;

        let res = execute(&fw, shader, &worker)?;

        let res_worker = Worker {
            file_name: worker.file_name,
            work_data: worker.work_data,
            out_data: res,
            work_size: worker.work_size,
        };

        let _ = receiver.send(res_worker).unwrap();
    }
}


fn execute<T>(fw: &Framework, shader: Shader, worker: &Worker<T>) -> Result<Vec<T>, Box<dyn Error>>
where
    T: GpuWorkType,
{
    let mut gpu_in_buffs: Vec<GpuBuffer<T>> = Vec::with_capacity(worker.work_data.len() + 1);

    for work in worker.work_data.iter() {
        let noe = GpuBuffer::from_slice(&fw, work);
        gpu_in_buffs.push(noe);
    }

    let gpu_out_buff = GpuBuffer::<T>::with_capacity(&fw, worker.out_data.len() as u64);

    // Build descriptor set base
    let mut descriptor_set = DescriptorSet::default();

    // Bind all input buffers
    for buff in gpu_in_buffs.iter() {
        descriptor_set = descriptor_set.bind_buffer(buff, GpuBufferUsage::ReadOnly);
    }
    // Bind output buffer
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

    use crate::utils::gpgpu::{worker::{Worker, Vec3}, gpu::gpu_task};

    #[test]
    pub fn test_gpgpu() -> Result<(), Box<dyn Error>> {
        let file_path = String::from("gpu/gpgpu_tests/compute.wgsl");

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

        let (left, right) = bichannel::channel::<Worker<u32>, Worker<u32>>();


        // let res = gpu_task_2(worker).await?;
        println!("Starting gpu task");
        // send right to gpu_task as another thread
        std::thread::spawn(move || {
            gpu_task(right).unwrap();
        });

        left.send(worker).unwrap();
        println!("Waiting for gpu task to finish");

        let res = left.recv().unwrap().out_data;
        // panic!("test");

        for (a, b) in cpu_computed_data.into_iter().zip(res) {
            // println!("{} {}", a, b);
            assert_eq!(a, b);
        }
        println!("Test passed");

        Ok(())
    }
}
