[[block]]
struct View {
    view_proj: mat4x4<f32>;
    projection: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

[[block]]
struct Mesh {
    transform: mat4x4<f32>;
};
[[group(2), binding(0)]]
var<uniform> mesh: Mesh;

struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    let world_position = mesh.transform * vec4<f32>(vertex.position, 1.0);

    var out: VertexOutput;
    out.clip_position = view.view_proj * world_position;
    out.uv = vertex.uv;

    return out;
}

[[block]]
struct CustomMaterial {
    color: vec4<f32>;
};
[[group(1), binding(0)]]
var<uniform> material: CustomMaterial;

[[group(1), binding(1)]]
var texture: texture_2d<f32>;
[[group(1), binding(2)]]
var sampler: sampler;


[[block]]
struct ViewSize {
    size: vec2<f32>;
};
[[group(3), binding(0)]]
var<uniform> view_size: ViewSize;

[[stage(fragment)]]
fn fragment(out: VertexOutput) -> [[location(0)]] vec4<f32> {
    let uv_view = vec2<f32>(out.clip_position.x / view_size.size.x, out.clip_position.y / view_size.size.y);

    let color = textureSample(texture, sampler, uv_view);
    return color;
}