use just_traits::scripting::{
    ScriptApiRegistry,
    FunctionResult,
    FunctionParameter,
    ParametersSource,
    function_params::*,
};

pub struct RenderableApi;

use crate::core::Renderable;

use crate::scripting::{MeshData, TextureData};

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
