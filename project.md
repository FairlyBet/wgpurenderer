# WGPURenderer Project Overview

A high-performance, modular WGPU-based rendering engine written in Rust. The project aims to provide a flexible and efficient abstraction over `wgpu` for modern graphics applications.

## Project Structure

The project is organized as a Rust workspace with two main members:

- **`wgpurenderer`**: The core library and rendering engine.
- **`wgpurenderer-macros`**: Procedural macros used to simplify engine interactions (e.g., immediate data handling).

## Core Architecture

### 1. Context & Renderer (`lib.rs`)

- **`Context`**: Wraps `wgpu::Instance`, `Adapter`, `Device`, and `Queue`. It handles the initialization of the graphics backend (Vulkan is prioritized).
- **`Renderer`**: The main interface for the engine. It manages:
  - Cache for `ShaderModule` and `BindGroupLayout`.
  - Swapchain/Surface management (`init_surface`, `resize`, `acquire`, `present`).
  - Pipeline creation with automatic layout derivation.
  - `ImmediateManager` for high-frequency data updates.

### 2. Rendering Pipeline

- **`RenderPass` (`renderpass.rs`)**: An abstraction for WGPU render passes, handling `DrawCall` execution.
- **`DrawCall`**: Encapsulates `Geometry`, `ShaderData` (bind groups + immediates), and instance count.
- **`Material`**: Defines the state of a rendering pipeline, including shaders, vertex layouts, and depth-stencil state.

### 3. Data Management

- **`ImmediateManager`**: Manages a shared byte buffer for "immediate" data (similar to push constants but via internal buffer management).
- **`SsboPool` (`ssbo.rs`)**: Manages Storage Buffer Objects (SSBOs) with staging buffers for efficient object-based data uploads.
- **`Transform` (`transform.rs`)**: Handles spatial transformations for scene nodes.

### 4. Scene & Camera

- **`Camera` (`camera.rs`)**: Implements view-projection logic using `glam`.
- **`Scene` / `Node`**: Basic structure for organizing renderable objects.

## Key Technologies

- **Graphics API**: [WGPU](https://wgpu.rs/) (WebGPU implementation for Rust).
- **Windowing**: [GLFW](https://www.glfw.org/) (via `glfw-rs`).
- **Math**: [glam](https://github.com/bitshifter/glam-rs) (SIMD-optimized linear algebra).
- **Serialization/Memory**: [bytemuck](https://github.com/Gilnaa/bytemuck) for zero-copy data handling.

## Development & Usage

### Prerequisites

- Rust toolchain (latest stable).
- Vulkan SDK/Drivers (recommended backend).

### Running the Project

The default runner is defined in `wgpurenderer/src/main.rs`.

```powershell
$env:RUST_LOG = "INFO"; cargo run
```

## Implementation Details for AI

- **Caching**: The engine aggressively caches `ShaderModule` and `BindGroupLayout` to minimize driver overhead.
- **Immediate Data**: Use the `immediate!` macro from `wgpurenderer-macros` to define structures that can be uploaded directly to the GPU via the `Immediate` system.
- **Memory Safety**: Leverages Rust's ownership model and `Rc`/`RefCell` for safe internal resource management where appropriate.
