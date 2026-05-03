use std::sync::Arc;
use winit::window::Window;
use crate::{SurfaceError, Error, Result};

pub struct Graphics
{
    #[allow(unused)]
    instance: wgpu::Instance,
    backend: wgpu::Backend,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    surface_size: Option<(u32, u32)>,
    view_format: wgpu::TextureFormat,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Graphics
{
    pub(crate) async fn init(backends: wgpu::Backends, features: wgpu::Features, limits: wgpu::Limits, window: Arc<Window>) -> Result<Self>
    {
        let instance_descr = wgpu::InstanceDescriptor
        {
            backends,
            flags: wgpu::InstanceFlags::from_build_config(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds
            {
                for_resource_creation: None,
                for_device_loss: None,
            },
            backend_options: wgpu::BackendOptions
            {
                gl: wgpu::GlBackendOptions
                {
                    gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
                    fence_behavior: wgpu::GlFenceBehavior::Normal,
                    #[cfg(debug_assertions)]
                    debug_fns: wgpu::GlDebugFns::Auto,
                    #[cfg(not(debug_assertions))]
                    debug_fns: wgpu::GlDebugFns::Disabled,
                },
                dx12: wgpu::Dx12BackendOptions
                {
                    shader_compiler: wgpu::Dx12Compiler::StaticDxc,
                    presentation_system: wgpu::Dx12SwapchainKind::DxgiFromHwnd,
                    latency_waitable_object: wgpu::Dx12UseFrameLatencyWaitableObject::None, //TODO expose this maybe?
                    force_shader_model: wgpu::ForceShaderModelToken::default(),
                    agility_sdk: None,
                },
                noop: wgpu::NoopBackendOptions
                {
                    enable: false,
                },
            },
            display: None,
        };
        let instance = wgpu::Instance::new(instance_descr);

        let surface = instance.create_surface(window)?;
        let surface_size = None;

        let adapter_opt = wgpu::RequestAdapterOptions
        {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        };
        let adapter = match instance.request_adapter(&adapter_opt).await
        {
            Ok(adapter) => adapter,
            Err(err) => return Err(Error::Adapter(err)), //err not Send+Sync on wasm -> no ? operator
        };
        let backend = adapter.get_info().backend;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or_else(|| surface_caps.formats[0]);
        let view_format = surface_format.add_srgb_suffix();

        let device_descr = wgpu::DeviceDescriptor
        {
            label: None,
            required_features: features,
            required_limits: limits,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        };
        let (device, queue) = match adapter.request_device(&device_descr).await
        {
            Ok(ok) => ok,
            Err(err) => return Err(Error::Device(err)), //err not Send+Sync on wasm -> no ? operator
        };

        Ok(Self { instance, backend, surface, surface_format, surface_size, view_format, device, queue })
    }

    pub(crate) fn configure(&mut self, (width, height): (u32, u32))
    {
        if width > 0 && height > 0 && Some((width, height)) != self.surface_size
        {
            self.surface_size = Some((width, height));
            let surface_conf = wgpu::SurfaceConfiguration
            {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                width, height,
                present_mode: wgpu::PresentMode::AutoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: if self.surface_format == self.view_format { vec![] } else { vec![self.view_format] },
            };
            self.surface.configure(&self.device, &surface_conf);
        }
    }

    pub fn backend(&self) -> wgpu::Backend { self.backend }
    pub fn view_format(&self) -> wgpu::TextureFormat { self.view_format }
    pub fn surface_size(&self) -> Option<(u32, u32)> { self.surface_size }

    pub fn current_surface(&mut self) -> Result<Option<(wgpu::SurfaceTexture, wgpu::TextureView)>>
    {
        let Some(size) = self.surface_size else { return Ok(None); };

        let texture = match self.surface.get_current_texture()
        {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(_) =>
            {
                self.configure(size);
                return Ok(None);
            },
            wgpu::CurrentSurfaceTexture::Timeout => return Ok(None),
            wgpu::CurrentSurfaceTexture::Occluded => return Ok(None),
            wgpu::CurrentSurfaceTexture::Outdated =>
            {
                self.configure(size);
                return Ok(None);
            },
            wgpu::CurrentSurfaceTexture::Lost => return Err(Error::Surface(SurfaceError::Lost)),
            wgpu::CurrentSurfaceTexture::Validation => return Err(Error::Surface(SurfaceError::Validation)),
        };
        let view_descr = wgpu::TextureViewDescriptor
        {
            label: None,
            format: Some(self.view_format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        };
        let view = texture.texture.create_view(&view_descr);

        Ok(Some((texture, view)))
    }
}
