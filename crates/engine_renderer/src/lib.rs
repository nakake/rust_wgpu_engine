use std::{iter, sync::Arc};
use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};

use engine_core::math::{Mat4, Vec3}; // Colorのimportは不要なので削除
use engine_ecs::{prelude::*, components::{Transform, Renderable}};

#[derive(Debug)]
pub enum RenderError {
    SurfaceLost,
    OutOfMemory,
}

// インスタンス描画用のデータ構造 (変更なし)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}

impl InstanceRaw {
    // VertexBufferLayoutの生成部分 (変更なし)
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        // ... この部分は前回と同じ ...
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 5, format: wgpu::VertexFormat::Float32x4, },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress, shader_location: 6, format: wgpu::VertexFormat::Float32x4, },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress, shader_location: 7, format: wgpu::VertexFormat::Float32x4, },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress, shader_location: 8, format: wgpu::VertexFormat::Float32x4, },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress, shader_location: 9, format: wgpu::VertexFormat::Float32x4, },
            ],
        }
    }
}


pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer, // index_bufferも保持するように変更
    instance_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        //【修正】Instance::newは引数を取るように変更された
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = instance.create_surface(window).unwrap();
        
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        //【修正】request_deviceの引数が1つになり、descriptorに新しいフィールドが追加
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::default(),
            },
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // Vertex buffer for a quad
        const VERTICES: &[[f32; 3]] = &[
            [-0.5, -0.5, 0.0],
            [0.5, -0.5, 0.0],
            [0.5, 0.5, 0.0],
            [-0.5, 0.5, 0.0],
        ];
        const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let num_indices = INDICES.len() as u32;

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        //【修正】RenderPipelineDescriptorに新しいフィールドが追加 & entry_pointがOptionになった
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // Someでラップ
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                    },
                    InstanceRaw::desc(),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"), // Someでラップ
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None, // 新しいフィールド
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<InstanceRaw>() * 1024) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            num_indices,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn render(&mut self, world: &mut World) -> Result<(), RenderError> {
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(wgpu::SurfaceError::Lost) => return Err(RenderError::SurfaceLost),
            Err(wgpu::SurfaceError::OutOfMemory) => return Err(RenderError::OutOfMemory),
            Err(e) => panic!("Unhandled wgpu surface error: {:?}", e), // とりあえず他のエラーはpanic
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut query = world.query::<(&Transform, &Renderable)>();
        let instance_data = query.iter(world).map(|(transform, renderable)| {
            let model_matrix = Mat4::from_scale(Vec3::new(transform.scale.x, transform.scale.y, 1.0))
                * Mat4::from_rotation_z(transform.rotation)
                * Mat4::from_translation(Vec3::new(transform.position.x, transform.position.y, 0.0));
            
            InstanceRaw {
                model: model_matrix.to_cols_array_2d(),
                color: [renderable.color.r, renderable.color.g, renderable.color.b, renderable.color.a],
            }
        }).collect::<Vec<_>>();

        self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instance_data));

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1, g: 0.2, b: 0.3, a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None, // 新しいwgpuでは明示的にNoneにするのが一般的
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..instance_data.len() as u32);
        }
    
        self.queue.submit(iter::once(encoder.finish()));
        output.present();
    
        Ok(())
    }
}