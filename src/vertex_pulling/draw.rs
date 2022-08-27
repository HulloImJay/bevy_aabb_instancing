use super::{
    buffer_cache::BufferCache,
    index_buffer::{num_indices_for_cuboids, CuboidsIndexBuffer},
    pipeline::CuboidsPipeline,
};

use bevy::{
    ecs::system::{lifetimeless::*, SystemParamItem},
    prelude::*,
    render::{
        render_phase::{
            EntityRenderCommand, PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass,
        },
        render_resource::{BindGroup, IndexFormat, PipelineCache},
        view::ViewUniformOffset,
    },
};

pub(crate) type DrawCuboids = (
    SetCuboidsPipeline,
    SetCuboidsViewBindGroup<0>,
    SetClippingPlanesBindGroup<1>,
    SetGpuTransformBufferBindGroup<2>,
    SetGpuCuboidBuffersBindGroup<3>,
    DrawVertexPulledCuboids,
);

pub(crate) struct SetCuboidsPipeline;

impl<P: PhaseItem> RenderCommand<P> for SetCuboidsPipeline {
    type Param = (SRes<PipelineCache>, SRes<CuboidsPipeline>);

    #[inline]
    fn render<'w>(
        _view: Entity,
        _item: &P,
        params: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (pipeline_cache, cuboids_pipeline) = params;
        if let Some(pipeline) = pipeline_cache
            .into_inner()
            .get_render_pipeline(cuboids_pipeline.pipeline_id)
        {
            pass.set_render_pipeline(pipeline);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

#[derive(Default)]
pub struct ViewMeta {
    pub cuboids_view_bind_group: Option<BindGroup>,
}

pub(crate) struct SetCuboidsViewBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetCuboidsViewBindGroup<I> {
    type Param = (SRes<ViewMeta>, SQuery<Read<ViewUniformOffset>>);
    #[inline]
    fn render<'w>(
        view: Entity,
        _item: Entity,
        (view_meta, view_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let view_uniform_offset = view_query.get(view).unwrap();
        pass.set_bind_group(
            I,
            view_meta
                .into_inner()
                .cuboids_view_bind_group
                .as_ref()
                .unwrap(),
            &[view_uniform_offset.offset],
        );

        RenderCommandResult::Success
    }
}

#[derive(Default)]
pub struct ClippingPlanesMeta {
    pub bind_group: Option<BindGroup>,
}

pub(crate) struct SetClippingPlanesBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetClippingPlanesBindGroup<I> {
    type Param = SRes<ClippingPlanesMeta>;
    #[inline]
    fn render<'w>(
        _view: Entity,
        _item: Entity,
        clipping_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(
            I,
            clipping_meta.into_inner().bind_group.as_ref().unwrap(),
            &[],
        );
        RenderCommandResult::Success
    }
}

pub(crate) struct SetGpuTransformBufferBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetGpuTransformBufferBindGroup<I> {
    type Param = SRes<BufferCache>;

    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        buffer_cache: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let buffer_cache = buffer_cache.into_inner();
        let entry = buffer_cache.get(item).unwrap();
        pass.set_bind_group(
            I,
            buffer_cache.transform_buffer_bind_group().unwrap(),
            &[entry.buffers().transform_index],
        );
        RenderCommandResult::Success
    }
}

pub(crate) struct SetGpuCuboidBuffersBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetGpuCuboidBuffersBindGroup<I> {
    type Param = SRes<BufferCache>;

    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        buffer_cache: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let entry = buffer_cache.into_inner().get(item).unwrap();
        pass.set_bind_group(I, &entry.buffers().instance_buffer_bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub(crate) struct DrawVertexPulledCuboids;

impl EntityRenderCommand for DrawVertexPulledCuboids {
    type Param = (SRes<BufferCache>, SRes<CuboidsIndexBuffer>);

    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (buffer_cache, index_buffer): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let entry = buffer_cache.into_inner().get(item).unwrap();
        let num_indices = num_indices_for_cuboids(entry.buffers().num_cuboids);
        pass.set_index_buffer(
            index_buffer.into_inner().buffer().unwrap().slice(..),
            0,
            IndexFormat::Uint32,
        );
        pass.draw_indexed(0..num_indices, 0, 0..1);
        RenderCommandResult::Success
    }
}