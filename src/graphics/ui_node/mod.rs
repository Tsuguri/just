use super::node_prelude::*;
use rendy::hal::{self, pass::Subpass, pso::CreationError};
use rendy::graph::render::{RenderGroup, RenderGroupDesc, PrepareResult};
use failure;
use legion::prelude::*;

pub struct UiNodeDesc<B: hal::Backend> {
    _mark: std::marker::PhantomData<B>,
}

impl<B: hal::Backend> std::default::Default for UiNodeDesc<B> {
    fn default() -> Self {
        Self {
            _mark: Default::default(),
        }
    }
}

pub struct UiNode<B: hal::Backend> {
    _mark: std::marker::PhantomData<B>,

}

struct UiRenderingData {
}

impl<B: hal::Backend> std::fmt::Debug for UiNodeDesc<B> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "UiNodeDesc")
    }
}

impl<B: hal::Backend> std::fmt::Debug for UiNode<B> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "UiNode")
    }
}

impl<B> RenderGroupDesc<B, World> for UiNodeDesc<B>
    where B: hal::Backend {
    fn build(
        self,
        ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        aux: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: Subpass<B>,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>
    ) -> Result<Box<dyn RenderGroup<B, World> + 'static>, failure::Error> {
        Result::Ok(
            Box::new(
                UiNode {
                    _mark: std::marker::PhantomData::default()
                }
            )
        )
    }
}

impl<B> RenderGroup<B, World> for UiNode<B>
    where B: hal::Backend {
    fn prepare(
            &mut self,
            factory: &Factory<B>,
            _queue: QueueId,
            index: usize,
            _subpass: hal::pass::Subpass<'_, B>,
            resources: &World,
        ) -> PrepareResult {
        PrepareResult::DrawReuse
    }
     fn draw_inline(
            &mut self,
            mut encoder: RenderPassEncoder<'_, B>,
            index: usize,
            _subpass: hal::pass::Subpass<'_, B>,
            _resources: &World,
        ) {
     }
    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &World) {
        /*unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }*/
    }
}
