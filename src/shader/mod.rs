//! Shader система с поддержкой модульной композиции
//!
//! # Примеры
//!
//! ## Создание шейдера из модулей:
//!
//! ```rust,ignore
//! use wgpurenderer::shader::{Shader, ShaderLayout, modules};
//!
//! let shader = Shader::from_modules(
//!     vec![
//!         modules::uniforms::CAMERA.into(),
//!         modules::uniforms::MODEL.into(),
//!         modules::vertex_inputs::STANDARD.into(),
//!         modules::vertex_outputs::STANDARD.into(),
//!         r#"
//!         @group(0) @binding(0) var<uniform> camera: CameraUniforms;
//!         @group(1) @binding(0) var<uniform> model: ModelUniforms;
//!
//!         @vertex
//!         fn vs_main(in: VertexInput) -> VertexOutput {
//!             var out: VertexOutput;
//!             let world_pos = model.model * vec4<f32>(in.position, 1.0);
//!             out.clip_position = camera.view_projection * world_pos;
//!             out.world_position = world_pos.xyz;
//!             out.normal = (model.normal_matrix * in.normal);
//!             out.uv = in.uv;
//!             return out;
//!         }
//!
//!         @fragment
//!         fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//!             return vec4<f32>(in.normal * 0.5 + 0.5, 1.0);
//!         }
//!         "#.into(),
//!     ],
//!     ShaderLayout::new()
//!         // ... layout description
//! );
//! ```
//!
//! ## Статические константы для переиспользования:
//!
//! ```rust,ignore
//! const MY_COMMON_CODE: &str = r#"
//! fn my_utility_function(x: f32) -> f32 {
//!     return x * 2.0;
//! }
//! "#;
//!
//! let shader1 = Shader::from_modules(
//!     vec![MY_COMMON_CODE.into(), /* ... */],
//!     layout1
//! );
//!
//! let shader2 = Shader::from_modules(
//!     vec![MY_COMMON_CODE.into(), /* ... */],  // Переиспользуем!
//!     layout2
//! );
//! ```

use std::path::{Path, PathBuf};

pub mod modules;

/// Основной тип шейдера - единый для встроенных и кастомных
pub struct Shader {
    /// WGSL исходный код
    source: ShaderSource,

    /// Entry points
    vertex_entry: String,
    fragment_entry: String,

    /// Описание требуемых данных (что шейдер ожидает)
    pub layout: ShaderLayout,
}

pub enum ShaderSource {
    /// Статический WGSL код (один кусок)
    Wgsl(String),

    /// Модульный WGSL код (несколько кусков, склеиваются при компиляции)
    WgslModules(Vec<Box<str>>),

    /// Загрузка из файла (lazy)
    File(PathBuf),

    /// Генерируемый код (для встроенных шейдеров с вариантами)
    Generated(Box<dyn Fn() -> String>),
}

#[derive(Debug)]
pub enum ShaderError {
    FileLoad(std::io::Error),
}

impl Shader {
    /// Кастомный шейдер из WGSL строки
    pub fn from_wgsl(source: &str, layout: ShaderLayout) -> Self {
        Self {
            source: ShaderSource::Wgsl(source.to_string()),
            vertex_entry: "vs_main".to_string(),
            fragment_entry: "fs_main".to_string(),
            layout,
        }
    }

    /// Кастомный шейдер из модулей (массив строк)
    pub fn from_modules(modules: Vec<Box<str>>, layout: ShaderLayout) -> Self {
        Self {
            source: ShaderSource::WgslModules(modules),
            vertex_entry: "vs_main".to_string(),
            fragment_entry: "fs_main".to_string(),
            layout,
        }
    }

    /// Кастомный шейдер из файла
    pub fn from_file(path: impl AsRef<Path>, layout: ShaderLayout) -> Self {
        Self {
            source: ShaderSource::File(path.as_ref().to_path_buf()),
            vertex_entry: "vs_main".to_string(),
            fragment_entry: "fs_main".to_string(),
            layout,
        }
    }

    /// С кастомными entry points
    pub fn with_entry_points(mut self, vertex: &str, fragment: &str) -> Self {
        self.vertex_entry = vertex.to_string();
        self.fragment_entry = fragment.to_string();
        self
    }

    /// Получить vertex entry point
    pub fn vertex_entry(&self) -> &str {
        &self.vertex_entry
    }

    /// Получить fragment entry point
    pub fn fragment_entry(&self) -> &str {
        &self.fragment_entry
    }

    /// Получить исходный код (загружается лениво если нужно)
    pub fn get_source(&self) -> Result<String, ShaderError> {
        match &self.source {
            ShaderSource::Wgsl(s) => Ok(s.clone()),
            ShaderSource::WgslModules(modules) => {
                // Склеиваем все модули в одну строку
                Ok(modules.iter().map(|s| s.as_ref()).collect::<Vec<_>>().join("\n"))
            }
            ShaderSource::File(path) => {
                std::fs::read_to_string(path).map_err(ShaderError::FileLoad)
            }
            ShaderSource::Generated(generator) => Ok(generator()),
        }
    }
}

/// Описание формата вершин
#[derive(Clone)]
pub struct VertexFormat {
    pub attributes: Vec<VertexAttribute>,
}

#[derive(Clone)]
pub struct VertexAttribute {
    pub location: u32,
    pub format: AttributeFormat,
    pub name: &'static str, // Для документации
}

#[derive(Clone, Copy, Debug)]
pub enum AttributeFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,
}

impl AttributeFormat {
    /// Размер атрибута в байтах
    pub fn size(&self) -> u64 {
        self.to_wgpu().size()
    }

    /// Конвертация в wgpu::VertexFormat
    pub fn to_wgpu(&self) -> wgpu::VertexFormat {
        match self {
            AttributeFormat::Float32 => wgpu::VertexFormat::Float32,
            AttributeFormat::Float32x2 => wgpu::VertexFormat::Float32x2,
            AttributeFormat::Float32x3 => wgpu::VertexFormat::Float32x3,
            AttributeFormat::Float32x4 => wgpu::VertexFormat::Float32x4,
            AttributeFormat::Uint32 => wgpu::VertexFormat::Uint32,
            AttributeFormat::Uint32x2 => wgpu::VertexFormat::Uint32x2,
            AttributeFormat::Uint32x3 => wgpu::VertexFormat::Uint32x3,
            AttributeFormat::Uint32x4 => wgpu::VertexFormat::Uint32x4,
        }
    }
}

/// Описание bind group layout
#[derive(Clone)]
pub struct BindGroupLayoutDescriptor {
    pub group: u32,
    pub bindings: Vec<BindingDescriptor>,
}

#[derive(Clone)]
pub struct BindingDescriptor {
    pub binding: u32,
    pub ty: BindingType,
    pub visibility: ShaderStages,
    pub name: &'static str, // Для документации
}

#[derive(Clone, Copy, Debug)]
pub enum BindingType {
    UniformBuffer,
    StorageBuffer { read_only: bool },
    Texture2D,
    TextureCube,
    Texture3D,
    Sampler,
    SamplerComparison,
}

#[derive(Clone, Copy, Debug)]
pub struct ShaderStages {
    pub vertex: bool,
    pub fragment: bool,
    pub compute: bool,
}

impl ShaderStages {
    pub const VERTEX: Self = Self {
        vertex: true,
        fragment: false,
        compute: false,
    };

    pub const FRAGMENT: Self = Self {
        vertex: false,
        fragment: true,
        compute: false,
    };

    pub const VERTEX_FRAGMENT: Self = Self {
        vertex: true,
        fragment: true,
        compute: false,
    };

    pub const COMPUTE: Self = Self {
        vertex: false,
        fragment: false,
        compute: true,
    };

    /// Конвертация в wgpu::ShaderStages
    pub fn to_wgpu(&self) -> wgpu::ShaderStages {
        let mut stages = wgpu::ShaderStages::empty();
        if self.vertex {
            stages |= wgpu::ShaderStages::VERTEX;
        }
        if self.fragment {
            stages |= wgpu::ShaderStages::FRAGMENT;
        }
        if self.compute {
            stages |= wgpu::ShaderStages::COMPUTE;
        }
        stages
    }
}

/// Описание layout шейдера
#[derive(Clone)]
pub struct ShaderLayout {
    pub vertex_format: VertexFormat,
    pub bind_groups: Vec<BindGroupLayoutDescriptor>,
}

impl ShaderLayout {
    pub fn new() -> Self {
        Self {
            vertex_format: VertexFormat { attributes: vec![] },
            bind_groups: vec![],
        }
    }

    pub fn vertex_attribute(
        mut self,
        location: u32,
        format: AttributeFormat,
        name: &'static str,
    ) -> Self {
        self.vertex_format.attributes.push(VertexAttribute {
            location,
            format,
            name,
        });
        self
    }

    pub fn bind_group(self, group: u32) -> BindGroupBuilder {
        BindGroupBuilder::new(self, group)
    }
}

impl Default for ShaderLayout {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder для bind group
pub struct BindGroupBuilder {
    layout: ShaderLayout,
    group: u32,
    bindings: Vec<BindingDescriptor>,
}

impl BindGroupBuilder {
    pub fn new(layout: ShaderLayout, group: u32) -> Self {
        Self {
            layout,
            group,
            bindings: vec![],
        }
    }

    pub fn uniform(mut self, binding: u32, visibility: ShaderStages, name: &'static str) -> Self {
        self.bindings.push(BindingDescriptor {
            binding,
            ty: BindingType::UniformBuffer,
            visibility,
            name,
        });
        self
    }

    pub fn storage(
        mut self,
        binding: u32,
        visibility: ShaderStages,
        read_only: bool,
        name: &'static str,
    ) -> Self {
        self.bindings.push(BindingDescriptor {
            binding,
            ty: BindingType::StorageBuffer { read_only },
            visibility,
            name,
        });
        self
    }

    pub fn texture(mut self, binding: u32, name: &'static str) -> Self {
        self.bindings.push(BindingDescriptor {
            binding,
            ty: BindingType::Texture2D,
            visibility: ShaderStages::FRAGMENT,
            name,
        });
        self
    }

    pub fn texture_cube(mut self, binding: u32, name: &'static str) -> Self {
        self.bindings.push(BindingDescriptor {
            binding,
            ty: BindingType::TextureCube,
            visibility: ShaderStages::FRAGMENT,
            name,
        });
        self
    }

    pub fn sampler(mut self, binding: u32, name: &'static str) -> Self {
        self.bindings.push(BindingDescriptor {
            binding,
            ty: BindingType::Sampler,
            visibility: ShaderStages::FRAGMENT,
            name,
        });
        self
    }

    pub fn sampler_comparison(mut self, binding: u32, name: &'static str) -> Self {
        self.bindings.push(BindingDescriptor {
            binding,
            ty: BindingType::SamplerComparison,
            visibility: ShaderStages::FRAGMENT,
            name,
        });
        self
    }

    pub fn build(mut self) -> ShaderLayout {
        self.layout.bind_groups.push(BindGroupLayoutDescriptor {
            group: self.group,
            bindings: self.bindings,
        });
        self.layout
    }
}

/// Предопределенные форматы вершин
pub mod vertex_formats {
    use super::*;

    /// Только позиция
    pub fn position_only() -> VertexFormat {
        VertexFormat {
            attributes: vec![VertexAttribute {
                location: 0,
                format: AttributeFormat::Float32x3,
                name: "position",
            }],
        }
    }

    /// Позиция + UV
    pub fn position_uv() -> VertexFormat {
        VertexFormat {
            attributes: vec![
                VertexAttribute {
                    location: 0,
                    format: AttributeFormat::Float32x3,
                    name: "position",
                },
                VertexAttribute {
                    location: 1,
                    format: AttributeFormat::Float32x2,
                    name: "uv",
                },
            ],
        }
    }

    /// Позиция + Нормаль
    pub fn position_normal() -> VertexFormat {
        VertexFormat {
            attributes: vec![
                VertexAttribute {
                    location: 0,
                    format: AttributeFormat::Float32x3,
                    name: "position",
                },
                VertexAttribute {
                    location: 1,
                    format: AttributeFormat::Float32x3,
                    name: "normal",
                },
            ],
        }
    }

    /// Стандартный формат: Позиция + Нормаль + UV
    pub fn standard() -> VertexFormat {
        VertexFormat {
            attributes: vec![
                VertexAttribute {
                    location: 0,
                    format: AttributeFormat::Float32x3,
                    name: "position",
                },
                VertexAttribute {
                    location: 1,
                    format: AttributeFormat::Float32x3,
                    name: "normal",
                },
                VertexAttribute {
                    location: 2,
                    format: AttributeFormat::Float32x2,
                    name: "uv",
                },
            ],
        }
    }

    /// Позиция + Нормаль + UV + Tangent
    pub fn full() -> VertexFormat {
        VertexFormat {
            attributes: vec![
                VertexAttribute {
                    location: 0,
                    format: AttributeFormat::Float32x3,
                    name: "position",
                },
                VertexAttribute {
                    location: 1,
                    format: AttributeFormat::Float32x3,
                    name: "normal",
                },
                VertexAttribute {
                    location: 2,
                    format: AttributeFormat::Float32x2,
                    name: "uv",
                },
                VertexAttribute {
                    location: 3,
                    format: AttributeFormat::Float32x4,
                    name: "tangent",
                },
            ],
        }
    }
}
