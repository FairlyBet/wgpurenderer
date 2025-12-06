struct Uniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    light_color: vec3<f32>,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    let world_position = uniforms.model * vec4<f32>(vertex.position, 1.0);
    out.world_position = world_position.xyz;
    
    let normal_matrix = mat3x3<f32>(
        uniforms.model[0].xyz,
        uniforms.model[1].xyz,
        uniforms.model[2].xyz,
    );
    out.normal = normalize(normal_matrix * vertex.normal);
    
    out.clip_position = uniforms.projection * uniforms.view * world_position;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Light direction (from top-right-front)
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    
    // Ambient lighting
    let ambient_strength = 0.3;
    let ambient = ambient_strength * uniforms.light_color;
    
    // Diffuse lighting
    let normal = normalize(in.normal);
    let diff = max(dot(normal, light_dir), 0.0);
    let diffuse = diff * uniforms.light_color;
    
    // Object base color
    let object_color = vec3<f32>(0.8, 0.3, 0.5);
    
    // Final color
    let result = (ambient + diffuse) * object_color;
    
    return vec4<f32>(result, 1.0);
}
