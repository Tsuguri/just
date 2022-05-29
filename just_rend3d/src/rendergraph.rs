use rend3::{graph::ReadyData, types::TextureFormat, Renderer};

pub(crate) struct RenderGraph {
    base_rendergraph: rend3_routine::base::BaseRenderGraph,
    pbr_routine: rend3_routine::pbr::PbrRoutine,
    tonemapping_routine: rend3_routine::tonemapping::TonemappingRoutine,
}

impl RenderGraph {
    pub fn new(renderer: &Renderer, output_fomat: TextureFormat) -> Self {
        let base_rendergraph = rend3_routine::base::BaseRenderGraph::new(&renderer);

        let mut data_core = renderer.data_core.lock();
        let pbr_routine = rend3_routine::pbr::PbrRoutine::new(&renderer, &mut data_core, &base_rendergraph.interfaces);
        drop(data_core);
        let tonemapping_routine =
            rend3_routine::tonemapping::TonemappingRoutine::new(&renderer, &base_rendergraph.interfaces, output_fomat);
        Self {
            base_rendergraph,
            pbr_routine,
            tonemapping_routine,
        }
    }

    pub fn prepare_graph<'a>(
        &'a self,
        graph: &mut rend3::graph::RenderGraph<'a>,
        ready: &ReadyData,
        resolution: glam::UVec2,
    ) {
        // Add the default rendergraph without a skybox
        self.base_rendergraph.add_to_graph(
            graph,
            ready,
            &self.pbr_routine,
            None,
            &self.tonemapping_routine,
            resolution,
            rend3::types::SampleCount::One,
            glam::Vec4::ZERO,
            glam::Vec4::new(0.10, 0.05, 0.10, 1.0), // Nice scene-referred purple
        );
    }
}
