use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem,
};

use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    pbr::{
        MaterialPipeline, MaterialPipelineKey, MeshPipelineKey, MeshUniform, RenderMaterials,
        SetMeshBindGroup, SetMeshViewBindGroup, PBR_SHADER_HANDLE,
    },
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            AsBindGroup, AsBindGroupError, BindGroup, BindGroupLayout, OwnedBindingResource,
            PipelineCache, RenderPipelineDescriptor, SpecializedMeshPipeline,
            SpecializedMeshPipelineError, SpecializedMeshPipelines,
        },
        renderer::RenderDevice,
        texture::FallbackImage,
        view::{ExtractedView, VisibleEntities},
        Extract, Render, RenderApp, RenderSet,
    },
    utils::{HashMap, HashSet},
};
use bevy_math::Vec4Swizzles;

use super::{easing::LodEasing, LOD_MATERIAL_SHADER_HANDLE};

#[derive(AsBindGroup, TypePath, Debug, Clone, TypeUuid)]
#[uuid = "8dba752b-f8a1-47ba-8d11-b569ca74526f"]
#[bind_group_data(LodMaterialKey)]
pub struct LodMaterial<const U: usize> {
    #[uniform(0)]
    pub size: UVec3,
    #[uniform(1)]
    pub max_lod: u32,
    #[uniform(2)]
    pub period: u32,
    pub easing: LodEasing,
    #[uniform(3)]
    pub buckets: [UVec4; 2],
}

#[derive(Clone, Component, Deref, ExtractComponent)]
pub struct WrappedMaterial<M: Material>(pub Handle<M>);

impl<M: Material> From<Handle<M>> for WrappedMaterial<M> {
    fn from(value: Handle<M>) -> Self {
        Self(value)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LodMaterialKey {
    size: UVec3,
    easing: LodEasing,
    max_lod: u32,
    period: u32,
    buckets: [UVec4; 2],
}

impl<const U: usize> From<&LodMaterial<U>> for LodMaterialKey {
    fn from(value: &LodMaterial<U>) -> Self {
        Self {
            size: value.size,
            easing: value.easing,
            max_lod: value.max_lod,
            period: value.period,
            buckets: value.buckets,
        }
    }
}

#[derive(Default)]
pub struct LodMaterialPlugin<const U: usize, M: Material>(PhantomData<M>);

impl<const U: usize, M: Material> Plugin for LodMaterialPlugin<U, M>
where
    M::Data: Clone + PartialEq + Eq + Hash,
{
    fn build(&self, app: &mut App) {
        app.add_asset::<LodMaterial<U>>().add_plugins((
            ExtractComponentPlugin::<Handle<LodMaterial<U>>>::extract_visible(),
            ExtractComponentPlugin::<WrappedMaterial<M>>::extract_visible(),
        ));

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Transparent3d, DrawLodMaterial<U, M>>()
                .init_resource::<ExtractedLodMaterials<U>>()
                .init_resource::<RenderLodMaterials<U>>()
                .init_resource::<SpecializedMeshPipelines<LodMaterialPipeline<U, M>>>()
                .add_systems(ExtractSchedule, extract_lod_materials::<U>)
                .add_systems(
                    Render,
                    (
                        prepare_lod_materials::<U, M>.in_set(RenderSet::Prepare),
                        queue_lod_material_meshes::<U, M>.in_set(RenderSet::Queue),
                    ),
                );
        }
    }

    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<LodMaterialPipeline<U, M>>();
        }
    }
}

pub struct LodMaterialPipelineKey<const U: usize, M: Material> {
    material_key: MaterialPipelineKey<M>,
    bind_group_data: <LodMaterial<U> as AsBindGroup>::Data,
}

impl<const U: usize, M: Material> Eq for LodMaterialPipelineKey<U, M> where M::Data: PartialEq {}

impl<const U: usize, M: Material> PartialEq for LodMaterialPipelineKey<U, M>
where
    M::Data: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.material_key == other.material_key && self.bind_group_data == other.bind_group_data
    }
}

impl<const U: usize, M: Material> Clone for LodMaterialPipelineKey<U, M>
where
    M::Data: Clone,
{
    fn clone(&self) -> Self {
        Self {
            material_key: self.material_key.clone(),
            bind_group_data: self.bind_group_data.clone(),
        }
    }
}

impl<const U: usize, M: Material> Hash for LodMaterialPipelineKey<U, M>
where
    M::Data: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.material_key.hash(state);
        self.bind_group_data.hash(state);
    }
}

#[derive(Resource)]
pub struct LodMaterialPipeline<const U: usize, M: Material> {
    material_pipeline: MaterialPipeline<M>,
    lod_layout: BindGroupLayout,
    vertex_shader: Handle<Shader>,
    fragment_shader: Handle<Shader>,
}

impl<const U: usize, M: Material> SpecializedMeshPipeline for LodMaterialPipeline<U, M>
where
    M::Data: Clone + PartialEq + Eq + Hash,
{
    type Key = LodMaterialPipelineKey<U, M>;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self
            .material_pipeline
            .specialize(key.material_key, layout)?;

        descriptor
            .vertex
            .shader_defs
            .push(key.bind_group_data.easing.into());

        // TODO: move this to a bind command
        descriptor.layout.insert(3, self.lod_layout.clone());

        descriptor.vertex.shader = self.vertex_shader.clone();
        descriptor.fragment.as_mut().unwrap().shader = self.fragment_shader.clone();

        if let Some(label) = &mut descriptor.label {
            *label = format!("lod_{}", *label).into();
        }

        Ok(descriptor)
    }
}

impl<const U: usize, M: Material> FromWorld for LodMaterialPipeline<U, M> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        Self {
            material_pipeline: world.resource::<MaterialPipeline<M>>().clone(),
            lod_layout: LodMaterial::<U>::bind_group_layout(render_device),
            vertex_shader: LOD_MATERIAL_SHADER_HANDLE.typed(),
            fragment_shader: PBR_SHADER_HANDLE.typed(),
        }
    }
}

type DrawLodMaterial<const U: usize, M> = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetInnerMaterialBindGroup<U, M, 1>,
    SetMeshBindGroup<2>,
    DrawMeshLod<U, M>,
);

/// Sets the bind group for a given [`Material`] at the configured `I` index.
pub struct SetInnerMaterialBindGroup<const U: usize, M: Material, const I: usize>(PhantomData<M>);

impl<P: PhaseItem, const U: usize, M: Material, const I: usize> RenderCommand<P>
    for SetInnerMaterialBindGroup<U, M, I>
{
    type Param = SRes<RenderMaterials<M>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<WrappedMaterial<M>>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        lod_material_handle: &'w WrappedMaterial<M>,
        materials: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let material = materials.into_inner().get(&lod_material_handle.0).unwrap();
        pass.set_bind_group(I, &material.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct DrawMeshLod<const U: usize, M: Material>(PhantomData<M>);

impl<P: PhaseItem, const U: usize, M: Material> RenderCommand<P> for DrawMeshLod<U, M> {
    type Param = (SRes<RenderAssets<Mesh>>, SRes<RenderLodMaterials<U>>);
    type ViewWorldQuery = Read<ExtractedView>;
    type ItemWorldQuery = (
        Read<MeshUniform>,
        Read<Handle<Mesh>>,
        Read<Handle<LodMaterial<U>>>,
    );

    #[inline]
    fn render<'w>(
        _item: &P,
        view: ROQueryItem<'w, Self::ViewWorldQuery>,
        (mesh_uniform, mesh_handle, material_handle): ROQueryItem<'w, Self::ItemWorldQuery>,
        (meshes, materials): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let prepared_material = match materials.into_inner().get(material_handle) {
            Some(material) => material,
            None => return RenderCommandResult::Failure,
        };
        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        let world_position =
            mesh_uniform.transform * (prepared_material.key.size.as_vec3() / 2.0).extend(1.0);

        let distance = (world_position.xyz() - view.transform.translation()).length();

        let lod = prepared_material.key.easing.calculate(
            distance,
            prepared_material.key.period,
            prepared_material.key.max_lod,
        ) - 0.25;

        let floor_lod = lod.floor() as usize;

        let end_index = prepared_material.key.buckets[floor_lod / 4][floor_lod % 4] as u32;

        pass.set_bind_group(3, &prepared_material.bind_group, &[]);

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                count: _,
                index_format,
            } => {
                // let end_index = prepared_material.key.end_index * 6;
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..(end_index * 6), 0, 0..1);
            }
            GpuBufferInfo::NonIndexed => {
                // let end_index = prepared_material.key.end_index;
                pass.draw(0..end_index, 0..1);
            }
        }

        RenderCommandResult::Success
    }
}

pub fn queue_lod_material_meshes<const U: usize, M: Material>(
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    lod_pipeline: Res<LodMaterialPipeline<U, M>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<LodMaterialPipeline<U, M>>>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    render_materials: Res<RenderMaterials<M>>,
    render_lod_materials: Res<RenderLodMaterials<U>>,
    material_meshes: Query<(
        &WrappedMaterial<M>,
        &Handle<LodMaterial<U>>,
        &Handle<Mesh>,
        &MeshUniform,
    )>,
    mut views: Query<(
        &ExtractedView,
        &VisibleEntities,
        &mut RenderPhase<Transparent3d>,
    )>,
) where
    M::Data: Clone + PartialEq + Eq + Hash,
{
    for (view, visible_entities, mut transparent_phase) in &mut views {
        let draw_lod = transparent_draw_functions
            .read()
            .id::<DrawLodMaterial<U, M>>();
        let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());
        let view_key = MeshPipelineKey::from_hdr(view.hdr) | msaa_key;

        let rangefinder = view.rangefinder3d();
        for visible_entity in &visible_entities.entities {
            if let Ok((wrapped_material_handle, lod_material_handle, mesh_handle, mesh_uniform)) =
                material_meshes.get(*visible_entity)
            {
                if let (Some(mesh), Some(lod_material), Some(material)) = (
                    render_meshes.get(mesh_handle),
                    render_lod_materials.get(lod_material_handle),
                    render_materials.get(&wrapped_material_handle.0),
                ) {
                    let mesh_key = view_key
                        | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                    let material_key = MaterialPipelineKey {
                        mesh_key,
                        bind_group_data: material.key.clone(),
                    };
                    let key = LodMaterialPipelineKey {
                        material_key,
                        bind_group_data: lod_material.key.clone(),
                    };

                    let pipeline = pipelines
                        .specialize(&pipeline_cache, &lod_pipeline, key, &mesh.layout)
                        .unwrap();

                    transparent_phase.add(Transparent3d {
                        entity: *visible_entity,
                        draw_function: draw_lod,
                        pipeline,
                        distance: rangefinder.distance(&mesh_uniform.transform),
                    });
                }
            }
        }
    }
}

pub struct PreparedLodMaterial<const U: usize> {
    pub bindings: Vec<OwnedBindingResource>,
    pub bind_group: BindGroup,
    pub key: <LodMaterial<U> as AsBindGroup>::Data,
}

#[derive(Resource)]
pub struct ExtractedLodMaterials<const U: usize> {
    extracted: Vec<(Handle<LodMaterial<U>>, LodMaterial<U>)>,
    removed: Vec<Handle<LodMaterial<U>>>,
}

impl<const U: usize> Default for ExtractedLodMaterials<U> {
    fn default() -> Self {
        Self {
            extracted: default(),
            removed: default(),
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct RenderLodMaterials<const U: usize>(
    pub HashMap<Handle<LodMaterial<U>>, PreparedLodMaterial<U>>,
);

impl<const U: usize> Default for RenderLodMaterials<U> {
    fn default() -> Self {
        Self(default())
    }
}

fn extract_lod_materials<const U: usize>(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<LodMaterial<U>>>>,
    assets: Extract<Res<Assets<LodMaterial<U>>>>,
) {
    let mut changed_assets = HashSet::default();
    let mut removed = Vec::new();
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                changed_assets.insert(handle.clone_weak());
            }
            AssetEvent::Removed { handle } => {
                changed_assets.remove(handle);
                removed.push(handle.clone_weak());
            }
        }
    }

    let mut extracted_assets = Vec::new();
    for handle in changed_assets.drain() {
        if let Some(asset) = assets.get(&handle) {
            extracted_assets.push((handle, asset.clone()));
        }
    }

    commands.insert_resource(ExtractedLodMaterials {
        extracted: extracted_assets,
        removed,
    });
}

struct PrepareNextFrameLodMaterials<const U: usize> {
    assets: Vec<(Handle<LodMaterial<U>>, LodMaterial<U>)>,
}

impl<const U: usize> Default for PrepareNextFrameLodMaterials<U> {
    fn default() -> Self {
        Self { assets: default() }
    }
}

fn prepare_lod_materials<const U: usize, M: Material>(
    mut prepare_next_frame: Local<PrepareNextFrameLodMaterials<U>>,
    mut extracted_assets: ResMut<ExtractedLodMaterials<U>>,
    mut render_materials: ResMut<RenderLodMaterials<U>>,
    render_device: Res<RenderDevice>,
    images: Res<RenderAssets<Image>>,
    fallback_image: Res<FallbackImage>,
    pipeline: Res<LodMaterialPipeline<U, M>>,
) {
    let queued_assets = mem::take(&mut prepare_next_frame.assets);
    for (handle, material) in queued_assets.into_iter() {
        match prepare_lod_material(
            &material,
            &render_device,
            &images,
            &fallback_image,
            &pipeline,
        ) {
            Ok(prepared_asset) => {
                render_materials.insert(handle, prepared_asset);
            }
            Err(AsBindGroupError::RetryNextUpdate) => {
                prepare_next_frame.assets.push((handle, material));
            }
        }
    }

    for removed in mem::take(&mut extracted_assets.removed) {
        render_materials.remove(&removed);
    }

    for (handle, material) in mem::take(&mut extracted_assets.extracted) {
        match prepare_lod_material(
            &material,
            &render_device,
            &images,
            &fallback_image,
            &pipeline,
        ) {
            Ok(prepared_asset) => {
                render_materials.insert(handle, prepared_asset);
            }
            Err(AsBindGroupError::RetryNextUpdate) => {
                prepare_next_frame.assets.push((handle, material));
            }
        }
    }
}

fn prepare_lod_material<const U: usize, M: Material>(
    lod_material: &LodMaterial<U>,
    render_device: &RenderDevice,
    images: &RenderAssets<Image>,
    fallback_image: &FallbackImage,
    pipeline: &LodMaterialPipeline<U, M>,
) -> Result<PreparedLodMaterial<U>, AsBindGroupError> {
    let prepared =
        lod_material.as_bind_group(&pipeline.lod_layout, render_device, images, fallback_image)?;

    Ok(PreparedLodMaterial {
        bindings: prepared.bindings,
        bind_group: prepared.bind_group,
        key: prepared.data,
    })
}
