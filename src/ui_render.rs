use super::graphics::Graphics;
use winit::window::Window;
use gru_misc::math::*;

const SHADER: wgpu::ShaderModuleDescriptor<'static> = wgpu::include_wgsl!("ui.wgsl");

#[repr(C, packed)]
struct Vertex
{
    position: Vec2,
	color: Vec4,
	tex_coords: Vec2,
    layer: i32,
}

pub struct RenderData
{
    bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buf: wgpu::Buffer,
    len_vertices: u64,
    index_buf: wgpu::Buffer,
    len_indices: u64,
    num_indices: u32,
    glyphs_version: Option<u64>,
    glyphs: wgpu::Texture,
    glyphs_view: wgpu::TextureView,
    glyphs_sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
}

impl RenderData
{
    fn create_pipeline(device: &wgpu::Device, view_format: wgpu::TextureFormat) -> (wgpu::BindGroupLayout, wgpu::RenderPipeline)
    {
        let bind_group_layout_descriptor_descr = wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                wgpu::BindGroupLayoutEntry
                {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture
                    {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry
                {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ]
        };
        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_descriptor_descr);

        let pipeline_layout_descr = wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        };
        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_descr);

        let ui_shader = device.create_shader_module(SHADER);
        let color_target_state = wgpu::ColorTargetState
        {
            format: view_format,
            blend: Some(wgpu::BlendState
            {
                color: wgpu::BlendComponent
                {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent
                {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            write_mask: wgpu::ColorWrites::ALL,
        };
        let color_target_state = Some(color_target_state);

        let render_pipeline_descr = wgpu::RenderPipelineDescriptor
        {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState
            {
                module: &ui_shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout
                {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4, 2 => Float32x2, 3 => Sint32]
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState
            {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState
            {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState
            {
                module: &ui_shader,
                entry_point: Some("fs_main"),
                targets: std::slice::from_ref(&color_target_state),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        };
        let render_pipeline = device.create_render_pipeline(&render_pipeline_descr);

        (bind_group_layout, render_pipeline)
    }

    fn create_buffers(device: &wgpu::Device, num_vertices: u64, num_indices: u64) -> (wgpu::Buffer, wgpu::Buffer)
    {
        let vertices_len = num_vertices * std::mem::size_of::<Vertex>() as u64;
        let vertex_buf_descr = wgpu::BufferDescriptor
        {
            label: None,
            size: vertices_len,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        };
        let vertex_buf = device.create_buffer(&vertex_buf_descr);

        let indices_len = num_indices * std::mem::size_of::<u16>() as u64;
        let index_buf_descr = wgpu::BufferDescriptor
        {
            label: None,
            size: indices_len,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
            mapped_at_creation: false,
        };
        let index_buf = device.create_buffer(&index_buf_descr);

        (vertex_buf, index_buf)
    }

    fn create_glyphs(graphics: &Graphics, data: Option<&Vec<Vec<u8>>>) -> (wgpu::Texture, wgpu::TextureView)
    {
        let mut layer_count = data.map(|layers| layers.len() as u32).unwrap_or(1);
        if graphics.backend() == wgpu::Backend::Gl { layer_count = layer_count.max(2); } //GL does not like TextureArray with 1 element

        let glyphs_descr = wgpu::TextureDescriptor
        {
            label: None,
            size: wgpu::Extent3d
            {
                width: gru_ui::paint::TEXTURE_SIZE,
                height: gru_ui::paint::TEXTURE_SIZE,
                depth_or_array_layers: layer_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let glyphs = match data
        {
            None => graphics.device.create_texture(&glyphs_descr),
            Some(layers) =>
            {
                use wgpu::util::DeviceExt;
                let layer_size = gru_ui::paint::TEXTURE_SIZE.pow(2) as usize;
                let mut data = vec![0; layer_count as usize * layer_size];
                for i in 0..layers.len()
                {
                    let a = i * layer_size;
                    let b = a + layer_size;
                    data[a..b].copy_from_slice(&layers[i]);
                }
                graphics.device.create_texture_with_data(&graphics.queue, &glyphs_descr, wgpu::util::TextureDataOrder::LayerMajor, &data)
            },
        };
        let view_descr = wgpu::TextureViewDescriptor
        {
            label: None,
            format: None,
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: Some(layer_count),
        };
        let glyphs_view = glyphs.create_view(&view_descr);

        (glyphs, glyphs_view)
    }

    fn create_bind_group(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout, glyphs_view: &wgpu::TextureView, sampler: &wgpu::Sampler) -> wgpu::BindGroup
    {
        let bind_group_descr = wgpu::BindGroupDescriptor
        {
            label: None,
            layout: bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(glyphs_view),
                },
                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ]
        };
        device.create_bind_group(&bind_group_descr)
    }

    pub(crate) fn new(graphics: &Graphics) -> Self
    {
        let (bind_group_layout, render_pipeline) = Self::create_pipeline(&graphics.device, graphics.view_format());
        let (vertex_buf, index_buf) = Self::create_buffers(&graphics.device, 1, 1);
        let (len_vertices, len_indices, num_indices) = (0, 0, 0);
        let glyphs_version = None;
        let (glyphs, glyphs_view) = Self::create_glyphs(graphics, None);
        let sampler_descr = wgpu::SamplerDescriptor
        {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        };
        let glyphs_sampler = graphics.device.create_sampler(&sampler_descr);
        let bind_group = Self::create_bind_group(&graphics.device, &bind_group_layout, &glyphs_view, &glyphs_sampler);

        Self { bind_group_layout, render_pipeline, vertex_buf, len_vertices, index_buf, len_indices, num_indices, glyphs_version, glyphs, glyphs_view, glyphs_sampler, bind_group }
    }

    pub fn update(&mut self, graphics: &Graphics, data: &gru_ui::paint::Frame)
    {
        if let Some(size) = graphics.surface_size() && data.new
        {
            let size = Vec2(size.0 as f32, size.1 as f32);
            //convert vertices
            let mut vertices = Vec::with_capacity(data.vertices.len());
            for vertex in data.vertices
            {
                let position = Vec2::from(vertex.position).component_div(size) * 2.0 - Vec2(1.0, 1.0);
                let position = position.component_mul(Vec2(1.0, -1.0)); //vulkan -> wgpu coordinates
                let color = vertex.color.to_normalized_linear().into();
                let (tex_coords, layer) = match vertex.tex_coords
                {
                    Some((u, v, l)) => ((u, v).into(), l as i32),
                    None => ((0.0, 0.0).into(), -1)
                };
                let vertex = Vertex { position, color, tex_coords, layer };
                vertices.push(vertex);
            }
            //create new buffer if too small
            if vertices.len() as u64 > self.vertex_buf.size() || data.indices.len() as u64 > self.index_buf.size()
            {
                let (vertex_buf, index_buf) = Self::create_buffers(&graphics.device, vertices.len() as u64, data.indices.len() as u64);
                self.vertex_buf = vertex_buf;
                self.index_buf = index_buf;
            }
            //fill buffer
            let vertex_bytes = unsafe
            {
                let ptr = vertices.as_ptr() as *const u8;
                std::slice::from_raw_parts(ptr, vertices.len() * std::mem::size_of::<Vertex>())
            };
            graphics.queue.write_buffer(&self.vertex_buf, 0, vertex_bytes);
            self.len_vertices = vertex_bytes.len() as u64;
            let index_bytes = unsafe
            {
                let ptr = data.indices.as_ptr() as *const u8;
                std::slice::from_raw_parts(ptr, data.indices.len() * std::mem::size_of::<u16>())
            };
            graphics.queue.write_buffer(&self.index_buf, 0, index_bytes);
            self.len_indices = index_bytes.len() as u64;
            self.num_indices = data.indices.len() as u32;
        }
        //update glyphs if new
        if self.glyphs_version != Some(data.font_version)
        {
            let (glyphs, glyphs_view) = Self::create_glyphs(graphics, Some(data.font_data));
            let bind_group = Self::create_bind_group(&graphics.device, &self.bind_group_layout, &glyphs_view, &self.glyphs_sampler);

            self.glyphs_version = Some(data.font_version);
            self.glyphs = glyphs;
            self.glyphs_view = glyphs_view;
            self.bind_group = bind_group;
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass)
    {
        if self.num_indices > 0
        {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buf.slice(0..self.len_vertices));
            render_pass.set_index_buffer(self.index_buf.slice(0..self.len_indices), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
    }
}

pub fn ui_config(window: &Window, scale: f32) -> gru_ui::UiConfig
{
    let size: (f32, f32) = window.inner_size().into();
    let display_scale_factor = window.scale_factor() as f32;
    gru_ui::UiConfig
    {
        size: size.into(),
        scale,
        display_scale_factor,
    }
}
