use bevy::{
    core::FloatOrd,
    core_pipeline::Transparent2d,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::RenderAssets,
        render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline},
        render_resource::{
            BlendState, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace, MultisampleState, PolygonMode,
            PrimitiveState, PrimitiveTopology, RenderPipelineCache, RenderPipelineDescriptor, SpecializedPipeline,
            SpecializedPipelines, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
            VertexStepMode, ShaderStages,
        },
        texture::BevyDefault,
        view::VisibleEntities,
        RenderApp, RenderStage,
    },
    sprite::{
        DrawMesh2d, Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform, SetMesh2dBindGroup,
        SetMesh2dViewBindGroup,
    },
};
use naga::ShaderStage;

/// This example shows how to manually render 2d items using "mid level render apis" with a custom pipeline for 2d meshes
/// It doesn't use the [`Material2d`] abstraction, but changes the vertex buffer to include vertex color
/// Check out the "mesh2d" example for simpler / higher level 2d meshes
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(CircleRenderPlugin)
        .add_startup_system(star)
        .run();
}

fn star(
    mut commands: Commands,
    // We will add a new Mesh for the star being created
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut star = Mesh::new(PrimitiveTopology::PointList);

    let mut v_pos = vec![[0.0, 0.0, 0.0]];
    for i in 0..10 {
        let a = std::f32::consts::FRAC_PI_2 - i as f32 * std::f32::consts::TAU / 10.0;
        let r = (1 - i % 2) as f32 * 100.0 + 100.0;
        v_pos.push([r * a.cos(), r * a.sin(), 0.0]);
    }
    // Set the position attribute
    star.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    let mut v_color = vec![[0.0, 0.0, 0.0, 1.0]];
    v_color.extend_from_slice(&[[1.0, 1.0, 0.0, 1.0]; 10]);
    star.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);

    // We can now spawn the entities for the star and the camera
    commands.spawn_bundle((
        // We use a marker component to identify the custom colored meshes
        ColoredMesh2d::default(),
        // The `Handle<Mesh>` needs to be wrapped in a `Mesh2dHandle` to use 2d rendering instead of 3d
        Mesh2dHandle(meshes.add(star)),
        // These other components are needed for 2d meshes to be rendered
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        ComputedVisibility::default(),
    ));
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d());
}

/// A marker component for colored 2d meshes
#[derive(Component, Default)]
pub struct ColoredMesh2d;

/// Custom pipeline for 2d meshes with vertex colors
pub struct ColoredMesh2dPipeline {
    /// this pipeline wraps the standard [`Mesh2dPipeline`]
    mesh2d_pipeline: Mesh2dPipeline,
}

impl FromWorld for ColoredMesh2dPipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
        }
    }
}

// We implement `SpecializedPipeline` to customize the default rendering from `Mesh2dPipeline`
impl SpecializedPipeline for ColoredMesh2dPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position and color
        let vertex_attributes = vec![
            // Position (GOTCHA! Vertex_Position isn't first in the buffer due to how Mesh sorts attributes (alphabetically))
            VertexAttribute {
                format: VertexFormat::Float32x3,
                // this offset is the size of the color attribute, which is stored first
                offset: 16,
                // position is available at location 0 in the shader
                shader_location: 0,
            },
            // Color
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 1,
            },
        ];
        // This is the sum of the size of position and color attributes (12 + 16 = 28)
        let vertex_array_stride = 28;

        RenderPipelineDescriptor {
            vertex: VertexState {
                // Use our custom shader
                shader: CIRCLE_VERT_HANDLE.typed::<Shader>(),
                entry_point: "main".into(),
                shader_defs: Vec::new(),
                // Use our custom vertex buffer
                buffers: vec![VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                // Use our custom shader
                shader: CIRCLE_FRAG_HANDLE.typed::<Shader>(),
                shader_defs: Vec::new(),
                entry_point: "main".into(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            // Use the two standard uniforms for 2d meshes
            layout: Some(vec![
                // Bind group 0 is the view uniform
                self.mesh2d_pipeline.view_layout.clone(),
                // Bind group 1 is the mesh uniform
                self.mesh2d_pipeline.mesh_layout.clone(),
            ]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("colored_mesh2d_pipeline".into()),
        }
    }
}

// This specifies how to render a colored 2d mesh
type DrawColoredMesh2d = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<1>,
    // Draw the mesh
    DrawMesh2d,
);

const CIRCLE_FRAG: &str = include_str!("../assets/shaders/particles.frag");
const CIRCLE_VERT: &str = include_str!("../assets/shaders/particles.vert");

/// Plugin that renders [`ColoredMesh2d`]s
pub struct CircleRenderPlugin;

/// Handle to the custom shader with a unique random ID
pub const CIRCLE_FRAG_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0);
pub const CIRCLE_VERT_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1);

impl Plugin for CircleRenderPlugin {
    fn build(&self, app: &mut App) {
        // Load our custom shader
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(CIRCLE_FRAG_HANDLE, Shader::from_glsl(CIRCLE_FRAG, ShaderStage::Fragment));
        shaders.set_untracked(CIRCLE_VERT_HANDLE, Shader::from_glsl(CIRCLE_VERT, ShaderStage::Vertex));

        // Register our custom draw function and pipeline, and add our render systems
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .add_render_command::<Transparent2d, DrawColoredMesh2d>()
            .init_resource::<ColoredMesh2dPipeline>()
            .init_resource::<SpecializedPipelines<ColoredMesh2dPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_colored_mesh2d)
            .add_system_to_stage(RenderStage::Queue, queue_colored_mesh2d);
    }
}

/// Extract the [`ColoredMesh2d`] marker component into the render app
pub fn extract_colored_mesh2d(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Query<(Entity, &ComputedVisibility), With<ColoredMesh2d>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, computed_visibility) in query.iter() {
        if !computed_visibility.is_visible {
            continue;
        }
        values.push((entity, (ColoredMesh2d,)));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

/// Queue the 2d meshes marked with [`ColoredMesh2d`] using our custom pipeline and draw function
#[allow(clippy::too_many_arguments)]
pub fn queue_colored_mesh2d(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    colored_mesh2d_pipeline: Res<ColoredMesh2dPipeline>,
    mut pipelines: ResMut<SpecializedPipelines<ColoredMesh2dPipeline>>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    colored_mesh2d: Query<(&Mesh2dHandle, &Mesh2dUniform), With<ColoredMesh2d>>,
    mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
) {
    if colored_mesh2d.is_empty() {
        return;
    }
    // Iterate each view (a camera is a view)
    for (visible_entities, mut transparent_phase) in views.iter_mut() {
        let draw_colored_mesh2d = transparent_draw_functions.read().get_id::<DrawColoredMesh2d>().unwrap();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Ok((mesh2d_handle, mesh2d_uniform)) = colored_mesh2d.get(*visible_entity) {
                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                    mesh2d_key |= Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);
                }

                let pipeline_id = pipelines.specialize(&mut pipeline_cache, &colored_mesh2d_pipeline, mesh2d_key);

                let mesh_z = mesh2d_uniform.transform.w_axis.z;
                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    draw_function: draw_colored_mesh2d,
                    pipeline: pipeline_id,
                    // The 2d render items are sorted according to their z value before rendering,
                    // in order to get correct transparency
                    sort_key: FloatOrd(mesh_z),
                    // This material is not batched
                    batch_range: None,
                });
            }
        }
    }
}
