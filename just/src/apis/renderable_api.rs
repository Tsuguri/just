use just_core::traits::scripting::{
    ScriptApiRegistry,
    FunctionResult,
    FunctionParameter,
    ParametersSource,
    function_params::*,
};

use just_assets::{AssetStorage, Handle};
use just_wgpu::Mesh;
use just_wgpu::Texture;
use just_wgpu::Renderable;

use std::sync::Arc;

pub struct RenderableApi;

#[derive(Copy, Clone)]
pub struct MeshData {
    pub handle: Handle<Mesh>,
}

#[derive(Copy, Clone)]
pub struct TextureData {
    pub handle: Handle<Texture>,
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

        registry.register_function("getMesh", Some(&resources_namespace), |args: (World, String)| -> Option<MeshData> {
            let handle = (*args.0).resources.get::<AssetStorage<Mesh>>().unwrap().get_handle(&args.1);
            handle.map(|x| MeshData{handle: x})
        });

        registry.register_function("getTexture", Some(&resources_namespace), |args: (World, String)| {
            let handle = (*args.0).resources.get::<AssetStorage<Texture>>().unwrap().get_handle(&args.1);
            handle.map(|x| TextureData{handle: x})
        });

        let renderable_type = registry
            .register_component::<Renderable, _>("Renderable", None, || Default::default())
            .unwrap();

        registry.register_native_type_property(
            &renderable_type,
            "mesh",
            Some(|args: ComponentThis<Renderable>| -> Option<MeshData> {
                args.mesh_handle.map(|handle| MeshData { handle })
            }),
            Some(|mut args: (ComponentThis<Renderable>, Option<MeshData>)| {
                args.0.mesh_handle = args.1.map(|x| x.handle);
            }),
        );

        registry.register_native_type_property(
            &renderable_type,
            "texture",
            Some(|args: ComponentThis<Renderable>| -> Option<TextureData> {
                args.texture_handle.map(|x| TextureData { handle: x })
            }),
            Some(
                |mut args: (ComponentThis<Renderable>, Option<TextureData>)| {
                    args.0.texture_handle = args.1.map(|x| x.handle);
                },
            ),
        );
    }
}
