use just_core::traits::scripting::{
    ScriptApiRegistry,
    FunctionResult,
    FunctionParameter,
    ParametersSource,
    function_params::*,
};

use std::sync::Arc;

pub struct RenderableApi;

use crate::core::Renderable;

use crate::traits::{MeshId, ResourceProvider, TextureId};

#[derive(Copy, Clone)]
pub struct MeshData {
    pub id: MeshId,
}

#[derive(Copy, Clone)]
pub struct TextureData {
    pub id: TextureId,
}

impl FunctionResult for MeshData {}
impl FunctionParameter for MeshData {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let nat = source.read_native()?;
        Result::Ok(*nat)
    }
}

impl FunctionResult for TextureData {}
impl FunctionParameter for TextureData {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let nat = source.read_native()?;
        Result::Ok(*nat)
    }
}

impl RenderableApi {
    pub fn register<SAR: ScriptApiRegistry>(registry: &mut SAR) {
        let resources_namespace = registry.register_namespace("Resources", None);

        registry.register_function("getMesh", Some(&resources_namespace), |mut args: (World, String)| -> Option<MeshData> {
            let obj = (*args.0).resources.get::<Arc<dyn ResourceProvider>>().unwrap().get_mesh(&args.1);
            obj.map(|x| MeshData{id: x})
        });

        registry.register_function("getTexture", Some(&resources_namespace), |mut args: (World, String)| {
            let obj = (*args.0).resources.get::<Arc<dyn ResourceProvider>>().unwrap().get_texture(&args.1);
            obj.map(|x| TextureData{id: x})
        });

        let renderable_type = registry
            .register_component::<Renderable, _>("Renderable", None, || Default::default())
            .unwrap();

        registry.register_native_type_property(
            &renderable_type,
            "mesh",
            Some(|args: ComponentThis<Renderable>| -> Option<MeshData> {
                args.mesh.map(|x| MeshData { id: x })
            }),
            Some(|mut args: (ComponentThis<Renderable>, Option<MeshData>)| {
                args.0.mesh = args.1.map(|x| x.id);
            }),
        );

        registry.register_native_type_property(
            &renderable_type,
            "texture",
            Some(|args: ComponentThis<Renderable>| -> Option<TextureData> {
                args.texture.map(|x| TextureData { id: x })
            }),
            Some(
                |mut args: (ComponentThis<Renderable>, Option<TextureData>)| {
                    args.0.texture = args.1.map(|x| x.id);
                },
            ),
        );
    }
}
