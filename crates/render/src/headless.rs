//! A GPU without a window, for tests and for rendering from the command line.

/// What can go wrong opening a GPU.
#[derive(Debug, thiserror::Error)]
pub enum HeadlessError {
    /// No adapter at all, not even a software one.
    #[error("no GPU adapter: {0}")]
    NoAdapter(String),
    /// The adapter refused to open a device.
    #[error("could not open the GPU: {0}")]
    Device(String),
}

/// Opens the first available GPU. `WGPU_BACKEND` and `WGPU_ADAPTER_NAME`
/// pick a backend or an adapter, as in every wgpu program; on a machine
/// without a GPU, Mesa's llvmpipe is a software Vulkan that works.
pub fn open_gpu() -> Result<(wgpu::Device, wgpu::Queue), HeadlessError> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    }))
    .map_err(|e| HeadlessError::NoAdapter(e.to_string()))?;
    log::info!("rendering on {:?}", adapter.get_info().name);
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("psito3d headless"),
        required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
        ..Default::default()
    }))
    .map_err(|e| HeadlessError::Device(e.to_string()))
}
