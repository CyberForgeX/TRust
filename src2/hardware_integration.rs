use rustacuda::prelude::*;
use rustacuda::memory::DeviceBuffer;
use rustacuda::function::{BlockSize, GridSize};
use std::error::Error;
use std::ffi::CString;
use std::fs;

#[cfg(feature = "cuda_support")]
impl CUDAGPU {
    pub fn compute(&self, ptx_path: &str) -> Result<(), Box<dyn Error>> {
        rustacuda::init(CudaFlags::empty())?;
        let device = Device::get_device(0)?;
        let _context = Context::create_and_push(ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO, device)?;

        // Dynamically find and load the PTX file
        let ptx_data = fs::read_to_string(ptx_path)?;
        let module = Module::load_from_string(&ptx_data)?;

        // Prepare your data and buffers
        let mut input = DeviceBuffer::from_slice(&[1.0f32; 1024])?;
        let mut output = DeviceBuffer::from_slice(&[0.0f32; 1024])?;

        // Create a CUDA stream
        let stream = Stream::new(StreamFlags::NON_BLOCKING, None)?;
        // Specify your kernel function name as defined in the PTX code
        let function_name = CString::new("your_kernel_function")?;
        let func = module.get_function(&function_name)?;
        let threads_per_block = 128;
        let block_count = 1024 / threads_per_block;

        // Launch the kernel
        unsafe {
            launch!(func<<<block_count, threads_per_block, 0, stream>>>(
                input.as_device_ptr(),
                output.as_device_ptr(),
                1024
            ))?;
        }

        // Synchronize the stream to wait for kernel completion
        stream.synchronize()?;

        // Optionally, copy the output back to host memory
        let mut host_output = [0.0f32; 1024];
        output.copy_to(&mut host_output)?;

        println!("Computation completed with CUDA");

        Ok(())
    }
}

