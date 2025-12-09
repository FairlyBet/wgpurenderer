/// Готовые модули шейдерного кода для переиспользования

/// Общие uniform структуры
pub mod uniforms {
    /// Стандартная структура для camera uniforms
    pub const CAMERA: &str = r#"
struct CameraUniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    position: vec3<f32>,
}
"#;

    /// Стандартная структура для model transform
    pub const MODEL: &str = r#"
struct ModelUniforms {
    model: mat4x4<f32>,
    normal_matrix: mat3x3<f32>,
}
"#;

    /// Стандартная структура для освещения
    pub const LIGHTING: &str = r#"
struct DirectionalLight {
    direction: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
}

struct PointLight {
    position: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
    range: f32,
}

struct LightingUniforms {
    ambient_color: vec3<f32>,
    ambient_intensity: f32,
    directional_light: DirectionalLight,
    num_point_lights: u32,
}
"#;

    /// PBR материал
    pub const PBR_MATERIAL: &str = r#"
struct PbrMaterial {
    base_color: vec4<f32>,
    metallic: f32,
    roughness: f32,
    emissive: vec3<f32>,
}
"#;
}

/// Vertex input структуры
pub mod vertex_inputs {
    /// Только позиция
    pub const POSITION: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
}
"#;

    /// Позиция + UV
    pub const POSITION_UV: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}
"#;

    /// Позиция + Нормаль + UV
    pub const STANDARD: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}
"#;

    /// Полный формат с tangent
    pub const FULL: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
}
"#;
}

/// Vertex output структуры
pub mod vertex_outputs {
    /// Минимальный output
    pub const MINIMAL: &str = r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}
"#;

    /// С UV
    pub const WITH_UV: &str = r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}
"#;

    /// Стандартный (для освещения)
    pub const STANDARD: &str = r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}
"#;

    /// Полный (с tangent для normal mapping)
    pub const FULL: &str = r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}
"#;
}

/// Utility функции
pub mod utils {
    /// Преобразование sRGB в linear
    pub const SRGB_TO_LINEAR: &str = r#"
fn srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
    return pow(srgb, vec3<f32>(2.2));
}
"#;

    /// Преобразование linear в sRGB
    pub const LINEAR_TO_SRGB: &str = r#"
fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    return pow(linear, vec3<f32>(1.0 / 2.2));
}
"#;

    /// Распаковка normal map
    pub const UNPACK_NORMAL: &str = r#"
fn unpack_normal(packed: vec3<f32>) -> vec3<f32> {
    return normalize(packed * 2.0 - 1.0);
}
"#;

    /// TBN матрица для normal mapping
    pub const CALCULATE_TBN: &str = r#"
fn calculate_tbn(normal: vec3<f32>, tangent: vec4<f32>) -> mat3x3<f32> {
    let n = normalize(normal);
    let t = normalize(tangent.xyz);
    let b = cross(n, t) * tangent.w;
    return mat3x3<f32>(t, b, n);
}
"#;
}

/// Освещение
pub mod lighting {
    /// Phong diffuse
    pub const PHONG_DIFFUSE: &str = r#"
fn phong_diffuse(normal: vec3<f32>, light_dir: vec3<f32>) -> f32 {
    return max(dot(normal, light_dir), 0.0);
}
"#;

    /// Phong specular
    pub const PHONG_SPECULAR: &str = r#"
fn phong_specular(
    normal: vec3<f32>,
    light_dir: vec3<f32>,
    view_dir: vec3<f32>,
    shininess: f32
) -> f32 {
    let reflect_dir = reflect(-light_dir, normal);
    return pow(max(dot(view_dir, reflect_dir), 0.0), shininess);
}
"#;

    /// Blinn-Phong specular
    pub const BLINN_PHONG_SPECULAR: &str = r#"
fn blinn_phong_specular(
    normal: vec3<f32>,
    light_dir: vec3<f32>,
    view_dir: vec3<f32>,
    shininess: f32
) -> f32 {
    let halfway_dir = normalize(light_dir + view_dir);
    return pow(max(dot(normal, halfway_dir), 0.0), shininess);
}
"#;

    /// Fresnel Schlick (для PBR)
    pub const FRESNEL_SCHLICK: &str = r#"
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(1.0 - cos_theta, 5.0);
}
"#;
}
