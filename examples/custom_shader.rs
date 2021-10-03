use bevy::core_pipeline::Transparent3d;
use bevy::ecs::prelude::*;
use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::SystemParamItem;
use bevy::math::{Vec2, Vec3, Vec4};
use bevy::pbr2::{DrawMesh, MeshUniform, PbrShaders, SetMeshViewBindGroup, SetTransformBindGroup};
use bevy::prelude::{AddAsset, App, Assets, GlobalTransform, Handle, Plugin, Transform};
use bevy::reflect::TypeUuid;
use bevy::render2::camera::PerspectiveCameraBundle;
use bevy::render2::color::Color;
use bevy::render2::mesh::{shape, Mesh};
use bevy::render2::render_asset::{
    PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets,
};
use bevy::render2::render_component::ExtractComponentPlugin;
use bevy::render2::render_phase::{
    AddRenderCommand, DrawFunctions, RenderCommand, RenderPhase, TrackedRenderPass,
};
use bevy::render2::render_resource::*;
use bevy::render2::renderer::{RenderDevice, RenderQueue};
use bevy::render2::shader::Shader;
use bevy::render2::texture::BevyDefault;
use bevy::render2::view::ExtractedView;
use bevy::render2::{RenderApp, RenderStage};
use bevy::PipelinedDefaultPlugins;
use bevy_portals::utils::Flycam;
use crevice::std140::{AsStd140, Std140};

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "4ee9c363-1124-4113-890e-199d81b00281"]
pub struct CustomMaterial {
    color: Color,
}

#[derive(Clone)]
pub struct GpuCustomMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

impl RenderAsset for CustomMaterial {
    type ExtractedAsset = CustomMaterial;
    type PreparedAsset = GpuCustomMaterial;
    type Param = (SRes<RenderDevice>, SRes<CustomShaders>);

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, custom_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let color: Vec4 = extracted_asset.color.as_rgba_linear().into();
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: color.as_std140().as_bytes(),
            label: None,
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
            layout: &custom_pipeline.material_layout,
        });

        Ok(GpuCustomMaterial {
            _buffer: buffer,
            bind_group,
        })
    }
}
pub struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<CustomMaterial>()
            .add_plugin(ExtractComponentPlugin::<Handle<CustomMaterial>>::default())
            .add_plugin(RenderAssetPlugin::<CustomMaterial>::default());
        app.sub_app(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<CustomShaders>()
            .init_resource::<CustomShaderMeta>()
            .init_resource::<ViewSizeUniforms>()
            .add_system_to_stage(RenderStage::Prepare, prepare_view_sizes)
            .add_system_to_stage(RenderStage::Queue, queue_view_sizes)
            .add_system_to_stage(RenderStage::Queue, queue_custom);
    }
}

fn main() {
    App::new()
        .add_plugins(PipelinedDefaultPlugins)
        .add_plugin(CustomMaterialPlugin)
        .add_plugin(bevy_portals::utils::FlycamPlugin)
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    // cube
    commands.spawn().insert_bundle((
        meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        Transform::from_xyz(0.0, 0.5, 0.0),
        GlobalTransform::default(),
        materials.add(CustomMaterial {
            color: Color::GREEN,
        }),
    ));

    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(Flycam);
}

pub struct CustomShaders {
    material_layout: BindGroupLayout,
    pipeline: RenderPipeline,

    view_size_layout: BindGroupLayout,
}

impl FromWorld for CustomShaders {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();
        let shader = Shader::from_wgsl(include_str!("../assets/custom.wgsl"));
        let shader_module = render_device.create_shader_module(&shader);

        let material_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(Vec4::std140_size_static() as u64),
                },
                count: None,
            }],
            label: None,
        });

        let view_size_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: BufferSize::new(ViewSize::std140_size_static() as u64),
                },
                count: None,
            }],
        });

        let pbr_pipeline = world.get_resource::<PbrShaders>().unwrap();

        let pipeline_layout = render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[
                &pbr_pipeline.view_layout,
                &material_layout,
                &pbr_pipeline.mesh_layout,
                &view_size_layout,
            ],
        });

        let pipeline = render_device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            vertex: VertexState {
                buffers: &[VertexBufferLayout {
                    array_stride: 32,
                    step_mode: InputStepMode::Vertex,
                    attributes: &[
                        // Position (GOTCHA! Vertex_Position isn't first in the buffer due to how Mesh sorts attributes (alphabetically))
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 12,
                            shader_location: 0,
                        },
                        // Normal
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 1,
                        },
                        // Uv
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 24,
                            shader_location: 2,
                        },
                    ],
                }],
                module: &shader_module,
                entry_point: "vertex",
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "fragment",
                targets: &[ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::One,
                            operation: BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrite::ALL,
                }],
            }),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            layout: Some(&pipeline_layout),
            multisample: MultisampleState::default(),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
        });

        CustomShaders {
            pipeline,
            material_layout,
            view_size_layout,
        }
    }
}

#[derive(Default)]
struct CustomShaderMeta {
    view_size_bind_group: Option<BindGroup>,
}

struct ViewSize {
    size: Vec2,
}
impl AsStd140 for ViewSize {
    type Std140Type = crevice::std140::Vec2;

    fn as_std140(&self) -> Self::Std140Type {
        crevice::std140::Vec2 {
            x: self.size.x,
            y: self.size.y,
        }
    }

    fn from_std140(val: Self::Std140Type) -> Self {
        ViewSize {
            size: Vec2::new(val.x, val.y),
        }
    }
}
struct ViewSizeUniformOffset(u32);

#[derive(Default)]
struct ViewSizeUniforms {
    pub uniforms: DynamicUniformVec<ViewSize>,
}
impl ViewSizeUniforms {
    fn push(&mut self, value: ViewSize) -> ViewSizeUniformOffset {
        let offset = self.uniforms.push(value);
        ViewSizeUniformOffset(offset)
    }
}

fn prepare_view_sizes(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut view_size_uniforms: ResMut<ViewSizeUniforms>,
    extracted_views: Query<(Entity, &ExtractedView)>,
) {
    view_size_uniforms
        .uniforms
        .reserve_and_clear(extracted_views.iter().len(), &render_device);

    for (entity, view) in extracted_views.iter() {
        let offset = view_size_uniforms.push(ViewSize {
            size: Vec2::new(view.width as f32, view.height as f32),
        });

        commands.entity(entity).insert(offset);
    }

    view_size_uniforms.uniforms.write_buffer(&render_queue);
}

fn queue_view_sizes(
    mut custom_shader_meta: ResMut<CustomShaderMeta>,
    custom_shaders: Res<CustomShaders>,
    view_size_uniforms: Res<ViewSizeUniforms>,
    render_device: Res<RenderDevice>,
) {
    let view_size_binding = match view_size_uniforms.uniforms.binding() {
        Some(val) => val,
        None => return,
    };

    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &custom_shaders.view_size_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: view_size_binding,
        }],
    });
    custom_shader_meta.view_size_bind_group = Some(bind_group);
}

pub fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    materials: Res<RenderAssets<CustomMaterial>>,
    material_meshes: Query<(Entity, &Handle<CustomMaterial>, &MeshUniform), With<Handle<Mesh>>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions
        .read()
        .get_id::<DrawCustom>()
        .unwrap();
    for (view, mut transparent_phase) in views.iter_mut() {
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);
        for (entity, material_handle, mesh_uniform) in material_meshes.iter() {
            if materials.contains_key(material_handle) {
                transparent_phase.add(Transparent3d {
                    entity,
                    draw_function: draw_custom,
                    distance: view_row_2.dot(mesh_uniform.transform.col(3)),
                });
            }
        }
    }
}

type DrawCustom = (
    SetCustomMaterialPipeline,
    SetMeshViewBindGroup<0>,
    SetTransformBindGroup<2>,
    SetViewSizesBindGroup<3>,
    DrawMesh,
);

struct SetCustomMaterialPipeline;

impl RenderCommand<Transparent3d> for SetCustomMaterialPipeline {
    type Param = (
        SRes<RenderAssets<CustomMaterial>>,
        SRes<CustomShaders>,
        SQuery<Read<Handle<CustomMaterial>>>,
    );
    fn render<'w>(
        _view: Entity,
        item: &Transparent3d,
        (materials, custom_pipeline, query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) {
        let material_handle = query.get(item.entity).unwrap();
        let material = materials.into_inner().get(material_handle).unwrap();
        pass.set_render_pipeline(&custom_pipeline.into_inner().pipeline);
        pass.set_bind_group(1, &material.bind_group, &[]);
    }
}

struct SetViewSizesBindGroup<const I: usize>;

impl<const I: usize> RenderCommand<Transparent3d> for SetViewSizesBindGroup<I> {
    type Param = (SRes<CustomShaderMeta>, SQuery<Read<ViewSizeUniformOffset>>);

    fn render<'w>(
        view: Entity,
        _: &Transparent3d,
        (meta, view_size_offsets): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) {
        let view_size_offset = view_size_offsets.get(view).unwrap();
        let view_size_bind_group = meta.into_inner().view_size_bind_group.as_ref().unwrap();

        pass.set_bind_group(I, view_size_bind_group, &[view_size_offset.0]);
    }
}
