use eframe::wgpu;
use encase::{ShaderSize, ShaderType};
use math::{Transform, Vector3};

mod color;

pub use color::*;

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct GpuCamera {
    pub transform: Transform,
    pub up_sky_color: Color,
    pub down_sky_color: Color,
    pub sun_color: Color,
    pub sun_direction: Vector3,
    pub sun_size: f32,
    pub recursive_portal_count: u32,
    pub max_bounces: u32,
}

pub const RENDER_TYPE_UNLIT: u32 = 0;
pub const RENDER_TYPE_LIT: u32 = 1;

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct GpuSceneInfo {
    pub camera: GpuCamera,
    pub aspect: f32,
    pub accumulated_frames: u32,
    pub random_seed: u32,
    pub render_type: u32,
    pub samples_per_pixel: u32,
    pub antialiasing: u32,
    pub plane_count: u32,
}

/// An XZ plane transformed by `transform`
#[derive(Debug, Clone, Copy, ShaderType)]
pub struct GpuPlane {
    pub transform: Transform,
    pub width: f32,
    pub height: f32,
    pub checker_count_x: u32,
    pub checker_count_z: u32,
    pub color: Color,
    pub checker_darkness: f32,
    pub emissive_color: Color,
    pub emissive_checker_darkness: f32,
    pub front_portal: GpuPortalConnection,
    pub back_portal: GpuPortalConnection,
}

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct GpuPortalConnection {
    /// u32::MAX is no connection
    pub other_index: u32,
    // pub flip: u32,
}

pub struct RayTracingRenderer {
    ray_tracing_texture: wgpu::Texture,
    ray_tracing_texture_write_bind_group_layout: wgpu::BindGroupLayout,
    ray_tracing_texture_sample_bind_group_layout: wgpu::BindGroupLayout,
    ray_tracing_texture_write_bind_group: wgpu::BindGroup,
    ray_tracing_texture_sample_bind_group: wgpu::BindGroup,

    full_screen_quad_pipeline: wgpu::RenderPipeline,

    scene_info_buffer: wgpu::Buffer,
    scene_info_bind_group: wgpu::BindGroup,

    planes_buffer: wgpu::Buffer,
    objects_bind_group_layout: wgpu::BindGroupLayout,
    objects_bind_group: wgpu::BindGroup,

    ray_tracing_pipeline: wgpu::ComputePipeline,
}

impl RayTracingRenderer {
    pub fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let full_screen_quad_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/full_screen_quad.wgsl"
        )));

        let ray_tracing_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/ray_tracing.wgsl"
        )));

        let ray_tracing_texture = Self::ray_tracing_texture(device, 1, 1);
        let ray_tracing_texture_write_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Ray Tracing Texture Write Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });
        let ray_tracing_texture_sample_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Ray Tracing Texture Sample Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });
        let (ray_tracing_texture_write_bind_group, ray_tracing_texture_sample_bind_group) =
            Self::ray_tracing_texture_bind_groups(
                device,
                &ray_tracing_texture_write_bind_group_layout,
                &ray_tracing_texture_sample_bind_group_layout,
                &ray_tracing_texture,
            );

        let full_screen_quad_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Full Screen Quad Pipeline Layout"),
                bind_group_layouts: &[&ray_tracing_texture_sample_bind_group_layout],
                push_constant_ranges: &[],
            });
        let full_screen_quad_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Full Screen Quad Pipeline"),
                layout: Some(&full_screen_quad_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &full_screen_quad_shader,
                    entry_point: Some("vertex"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &full_screen_quad_shader,
                    entry_point: Some("fragment"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview: None,
                cache: None,
            });

        let scene_info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene Info Buffer"),
            size: GpuSceneInfo::SHADER_SIZE.get(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let scene_info_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Scene Info Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuSceneInfo::SHADER_SIZE),
                    },
                    count: None,
                }],
            });
        let scene_info_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene Info Bind Group Layout"),
            layout: &scene_info_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_info_buffer.as_entire_binding(),
            }],
        });

        let planes_buffer = Self::planes_buffer(device, GpuPlane::SHADER_SIZE.get());
        let objects_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Objects Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuPlane::SHADER_SIZE),
                    },
                    count: None,
                }],
            });
        let objects_bind_group =
            Self::objects_bind_group(device, &objects_bind_group_layout, &planes_buffer);

        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[
                    &ray_tracing_texture_write_bind_group_layout,
                    &scene_info_bind_group_layout,
                    &objects_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let ray_tracing_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Ray Tracing Pipeline"),
                layout: Some(&ray_tracing_pipeline_layout),
                module: &ray_tracing_shader,
                entry_point: Some("ray_trace"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        Self {
            ray_tracing_texture,
            ray_tracing_texture_write_bind_group_layout,
            ray_tracing_texture_sample_bind_group_layout,
            ray_tracing_texture_write_bind_group,
            ray_tracing_texture_sample_bind_group,

            full_screen_quad_pipeline,

            scene_info_buffer,
            scene_info_bind_group,

            planes_buffer,
            objects_bind_group_layout,
            objects_bind_group,

            ray_tracing_pipeline,
        }
    }

    fn planes_buffer(device: &wgpu::Device, size: wgpu::BufferAddress) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Planes Buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn objects_bind_group(
        device: &wgpu::Device,
        objects_bind_group_layout: &wgpu::BindGroupLayout,
        planes_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Objects Bind Group"),
            layout: objects_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: planes_buffer.as_entire_binding(),
            }],
        })
    }

    fn ray_tracing_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Ray Tracing Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }

    fn ray_tracing_texture_bind_groups(
        device: &wgpu::Device,
        ray_tracing_texture_write_bind_group_layout: &wgpu::BindGroupLayout,
        ray_tracing_texture_sample_bind_group_layout: &wgpu::BindGroupLayout,
        ray_tracing_texture: &wgpu::Texture,
    ) -> (wgpu::BindGroup, wgpu::BindGroup) {
        let ray_tracing_texture_view = ray_tracing_texture.create_view(&Default::default());
        let ray_tracing_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Ray Tracing Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let ray_tracing_texture_write_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Ray Tracing Texture Write Bind Group"),
                layout: ray_tracing_texture_write_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&ray_tracing_texture_view),
                }],
            });
        let ray_tracing_texture_sample_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Ray Tracing Texture Sample Bind Group"),
                layout: ray_tracing_texture_sample_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&ray_tracing_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&ray_tracing_texture_sampler),
                    },
                ],
            });
        (
            ray_tracing_texture_write_bind_group,
            ray_tracing_texture_sample_bind_group,
        )
    }
}

pub struct RayTracingPaintCallback {
    pub width: u32,
    pub height: u32,
    pub camera: GpuCamera,
    pub accumulated_frames: u32,
    pub random_seed: u32,
    pub render_type: u32,
    pub samples_per_pixel: u32,
    pub antialiasing: bool,
    pub planes: Vec<GpuPlane>,
}

impl eframe::egui_wgpu::CallbackTrait for RayTracingPaintCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let renderer: &mut RayTracingRenderer = callback_resources.get_mut().unwrap();

        {
            let ray_tracing_texture_size = renderer.ray_tracing_texture.size();
            if self.width > 0
                && self.height > 0
                && (ray_tracing_texture_size.width != self.width
                    || ray_tracing_texture_size.height != self.height)
            {
                renderer.ray_tracing_texture =
                    RayTracingRenderer::ray_tracing_texture(device, self.width, self.height);
                (
                    renderer.ray_tracing_texture_write_bind_group,
                    renderer.ray_tracing_texture_sample_bind_group,
                ) = RayTracingRenderer::ray_tracing_texture_bind_groups(
                    device,
                    &renderer.ray_tracing_texture_write_bind_group_layout,
                    &renderer.ray_tracing_texture_sample_bind_group_layout,
                    &renderer.ray_tracing_texture,
                );
            }
        }

        {
            let scene_info = GpuSceneInfo {
                camera: self.camera,
                aspect: self.width as f32 / self.height as f32,
                accumulated_frames: self.accumulated_frames,
                random_seed: self.random_seed,
                render_type: self.render_type,
                samples_per_pixel: self.samples_per_pixel,
                antialiasing: self.antialiasing as u32,
                plane_count: self.planes.len() as _,
            };

            let mut scene_info_buffer = queue
                .write_buffer_with(&renderer.scene_info_buffer, 0, GpuSceneInfo::SHADER_SIZE)
                .unwrap();
            encase::UniformBuffer::new(&mut *scene_info_buffer)
                .write(&scene_info)
                .unwrap();
        }

        {
            let mut should_recreate_objects_bind_group = false;

            {
                let size = self.planes.size();

                if size.get() > renderer.planes_buffer.size() {
                    renderer.planes_buffer = RayTracingRenderer::planes_buffer(device, size.get());
                    should_recreate_objects_bind_group = true;
                }

                let mut planes_buffer = queue
                    .write_buffer_with(&renderer.planes_buffer, 0, size)
                    .unwrap();
                encase::StorageBuffer::new(&mut *planes_buffer)
                    .write(&self.planes)
                    .unwrap();
            }

            if should_recreate_objects_bind_group {
                renderer.objects_bind_group = RayTracingRenderer::objects_bind_group(
                    device,
                    &renderer.objects_bind_group_layout,
                    &renderer.planes_buffer,
                );
            }
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Ray Tracing Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Ray Tracing Compute Pass"),
                timestamp_writes: None,
            });

            let ray_tracing_texture_size = renderer.ray_tracing_texture.size();

            compute_pass.set_pipeline(&renderer.ray_tracing_pipeline);
            compute_pass.set_bind_group(0, &renderer.ray_tracing_texture_write_bind_group, &[]);
            compute_pass.set_bind_group(1, &renderer.scene_info_bind_group, &[]);
            compute_pass.set_bind_group(2, &renderer.objects_bind_group, &[]);
            compute_pass.dispatch_workgroups(
                ray_tracing_texture_size.width.div_ceil(16),
                ray_tracing_texture_size.height.div_ceil(16),
                1,
            );
        }

        vec![encoder.finish()]
    }

    fn paint(
        &self,
        _info: eframe::egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        let renderer: &RayTracingRenderer = callback_resources.get().unwrap();

        render_pass.set_pipeline(&renderer.full_screen_quad_pipeline);
        render_pass.set_bind_group(0, &renderer.ray_tracing_texture_sample_bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}
