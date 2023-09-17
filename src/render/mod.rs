pub mod material;
pub mod easing;

use bevy::{asset::load_internal_asset, prelude::*, reflect::TypeUuid};

pub const LOD_BINDINGS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1983927262504844127);

pub const LOD_FUNCTIONS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2172304243506364900);

pub const LOD_MATERIAL_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5474541954525159662);

pub struct LodRenderPlugin;

impl Plugin for LodRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            LOD_BINDINGS_SHADER_HANDLE,
            "lod_bindings.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            LOD_FUNCTIONS_SHADER_HANDLE,
            "lod_functions.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            LOD_MATERIAL_SHADER_HANDLE,
            "material.wgsl",
            Shader::from_wgsl
        );
    }
}
