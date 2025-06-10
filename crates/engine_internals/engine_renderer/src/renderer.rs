use std::iter;
use wgpu::util::DeviceExt;

use crate::{instance::InstanceRaw, pipeline};
use engine_core::math::{Mat4, Vec3};
use engine_ecs::{
    components::{Renderable, Transform},
    prelude::*,
};

#[derive(Debug)]
pub enum RenderError {}

pub struct Renderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor::default());
        let render_pipeline = pipeline::create_render_pipeline(
            device,
            &render_pipeline_layout,
            &shader,
            config.format,
        );

        const VERTICES: &[[f32; 3]] = &[
            [-0.5, -0.5, 0.0],
            [0.5, -0.5, 0.0],
            [0.5, 0.5, 0.0],
            [-0.5, 0.5, 0.0],
        ];
        const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<InstanceRaw>() * 1024) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            num_indices,
        }
    }

    pub fn render(
        &mut self,
        world: &mut World,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), RenderError> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let mut query = world.query::<(&Transform, &Renderable)>();
        let instance_data = query
            .iter(world)
            .map(|(transform, renderable)| {
                let model_matrix = Mat4::from_translation(Vec3::new(
                    transform.position.x,
                    transform.position.y,
                    0.0,
                )) * Mat4::from_rotation_z(transform.rotation)
                    * Mat4::from_scale(Vec3::new(transform.scale.x, transform.scale.y, 1.0));
                InstanceRaw {
                    model: model_matrix.to_cols_array_2d(),
                    color: [
                        renderable.color.r,
                        renderable.color.g,
                        renderable.color.b,
                        renderable.color.a,
                    ],
                }
            })
            .collect::<Vec<_>>();

        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            if !instance_data.is_empty() {
                render_pass.draw_indexed(0..self.num_indices, 0, 0..instance_data.len() as u32);
            }
        }

        queue.submit(iter::once(encoder.finish()));
        Ok(())
    }
}
