# AI Context: WGPURenderer

This file provides context and instructions for AI models working on the `wgpurenderer` project.

## Core Mandates

1. **Strict Performance Focus**: The engine is intended for high-performance rendering. Avoid cloning large buffers or making redundant API calls.
2. **Explicit Resource Ownership**: Use Rust's ownership model to manage GPU resources. Follow existing patterns of caching `ShaderModule` and `BindGroupLayout`.
3. **Adhere to Internal Abstractions**:
    - Use `Immediate` system for per-draw-call small data updates.
    - Use `SsboPool` for larger storage buffer objects that change less frequently.
    - Use `RenderPass` for grouping draw calls.
4. **Backend Target**: Prioritize Vulkan features while maintaining WGPU compatibility.

## Coding Style & Patterns

- **Line Endings**: Always use LF (Unix-style) line endings. This is enforced by `.gitattributes` and `rustfmt.toml`.
- **Error Handling**: Use `Option` and `Result` where appropriate. Internal panics (`utils::cold_panic`) are only for unrecoverable logic errors.
- **Macros**: Use `wgpurenderer_macros::immediate!` to define data structures that match shader storage layouts.
- **Math**: Use `glam` types (`Mat4`, `Vec2`, `Vec3`, `Quat`) for all linear algebra.
- **Data Layouts**: Ensure all structs intended for GPU upload are `#[repr(C)]` and implement `bytemuck::Pod` / `bytemuck::Zeroable`.

## Common Tasks & Snippets

### Defining Immediate Data

```rust
use wgpurenderer_macros::immediate;

#[immediate]
struct MyShaderData {
    model_matrix: glam::Mat4,
    color: glam::Vec4,
}
```

### Creating a Draw Call

1. Define `Material`.
2. Create `Geometry` (vertex/index buffers).
3. Allocate `Immediate` space if needed.
4. Construct `DrawCall`.

## Documentation & Comments

- Maintain internal documentation for all `pub` items in `lib.rs`.
- Use "TODO" comments for planned refactorings (e.g., pipeline caching, buffer management).

## Project Roadmap

- [ ] Implement robust `RenderPipeline` caching in `Renderer`.
- [ ] Complete `SsboPool` memory management (freeing buffers).
- [ ] Add support for depth-testing and stencil operations.
- [ ] Integrate a scene graph for complex transformations.
