use std::sync::Arc;
use anyhow::Result;
use winit::window::Window;

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
    pub(crate) async fn init(backends: wgpu::Backends, window: Arc<Window>) -> Result<Self>
    {
        let instance_descr = wgpu::InstanceDescriptor
        {
            backends,
            flags: wgpu::InstanceFlags::from_build_config(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Dxc { dxil_path: None, dxc_path: None },
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
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
            Some(adapter) => adapter,
            None => anyhow::bail!("No compatible GPU found."),
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
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
            memory_hints: wgpu::MemoryHints::Performance,
        };
        let (device, queue) = match adapter.request_device(&device_descr, None).await
        {
            Ok(ok) => ok,
            Err(err) => anyhow::bail!("{err:?}"), //err not Send+Sync on wasm -> no ? operator
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
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) =>
            {
                self.configure(size);
                return Ok(None);
            },
            Err(err) => anyhow::bail!("{err:?}"),
        };
        let view_descr = wgpu::TextureViewDescriptor
        {
            label: None,
            format: Some(self.view_format),
            dimension: Some(wgpu::TextureViewDimension::D2),
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
