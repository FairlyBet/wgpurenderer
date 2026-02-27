use crate::{DrawCall, RenderTarget};
use smallvec::SmallVec;
use std::{fmt::Debug, num::NonZeroU32};

#[derive(Debug)]
pub struct RenderPass {
    pub render_target: RenderTarget,
    // TODO: pub timestamp_writes: Option<RenderPassTimestampWrites<'a>>,
    // TODO: pub occlusion_query_set: Option<&'a QuerySet>,
    pub multiview_mask: Option<NonZeroU32>,
    pub draw_calls: Vec<DrawCall>,
    pub executor: Option<Box<dyn RenderPassExecutor>>,
}

impl RenderPass {
    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let color_attachments: SmallVec<[_; 1]> = self
            .render_target
            .color_attachments
            .iter()
            .map(|attachment| {
                Some(wgpu::RenderPassColorAttachment {
                    view: &attachment.view,
                    resolve_target: attachment.resolve_target.as_ref(),
                    ops: attachment.ops,
                    depth_slice: attachment.depth_slice,
                })
            })
            .collect();

        let depth_stencil_attachment =
            self.render_target.depth_stencil_attachment.as_ref().map(|attachment| {
                wgpu::RenderPassDepthStencilAttachment {
                    view: &attachment.view,
                    depth_ops: attachment.depth_ops,
                    stencil_ops: attachment.stencil_ops,
                }
            });

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: self.multiview_mask,
        };

        if let Some(executor) = &mut self.executor {
            executor.execute(encoder, &render_pass_descriptor);
        } else {
            let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);
            execute_ordered_draw_calls(&mut render_pass, &mut self.draw_calls);
        }
    }
}

pub trait RenderPassExecutor: Debug {
    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_pass_descriptor: &wgpu::RenderPassDescriptor,
    );
}

pub fn execute_ordered_draw_calls(render_pass: &mut wgpu::RenderPass, draw_calls: &mut [DrawCall]) {
    // Sort draw calls to minimize state changes: Pipeline -> BindGroups
    draw_calls.sort_by(|a, b| match a.render_pipeline_handle.cmp(&b.render_pipeline_handle) {
        std::cmp::Ordering::Equal => {
            a.shader_data.bind_groups.iter().cmp(b.shader_data.bind_groups.iter())
        }
        ord => ord,
    });

    let mut current_pipeline_id = None;
    let mut current_bind_groups: SmallVec<[Option<&wgpu::BindGroup>; 3]> =
        SmallVec::from_elem(None, 3);

    for draw_call in draw_calls {
        // 1. Set pipeline
        if current_pipeline_id != Some(&draw_call.render_pipeline_handle) {
            render_pass.set_pipeline(&draw_call.render_pipeline_handle);
            current_pipeline_id = Some(&draw_call.render_pipeline_handle);
            // Reset bind groups cache because new pipeline might have different layouts
            current_bind_groups.fill(None);
        }

        // 2. Set bind groups
        for (i, bind_group) in draw_call.shader_data.bind_groups.iter().enumerate() {
            if i >= current_bind_groups.len()
                || current_bind_groups[i].is_none_or(|b| *b != *bind_group)
            {
                render_pass.set_bind_group(i as u32, bind_group, &[]);

                if i < current_bind_groups.len() {
                    current_bind_groups[i] = Some(bind_group);
                }
            }
        }

        // 3. Set vertex/index buffers
        for (i, (buffer, range)) in draw_call.geometry.buffers.iter().enumerate() {
            let start = range.as_ref().map_or(0, |r| r.start as u64);
            let end = range.as_ref().map_or(buffer.size(), |r| r.end as u64);
            render_pass.set_vertex_buffer(i as u32, buffer.slice(start..end));
        }

        if !draw_call.shader_data.immediates.is_empty() {
            render_pass.set_immediates(0, &draw_call.shader_data.immediates);
        }

        if let Some(index_buffer) = &draw_call.geometry.index_buffer {
            render_pass.set_index_buffer(index_buffer.slice(..), draw_call.geometry.index_format);
            render_pass.draw_indexed(
                0..draw_call.geometry.count,
                0,
                0..draw_call.instance_count.get(),
            );
        } else {
            render_pass.draw(0..draw_call.geometry.count, 0..draw_call.instance_count.get());
        }
    }
}
