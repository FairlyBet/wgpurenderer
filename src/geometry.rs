use crate::shader::VertexAttribute;

/// Один vertex буфер с описанием атрибутов
pub struct VertexBuffer {
    buffer: wgpu::Buffer,
    slot: u32,
    attributes: Vec<VertexAttribute>,
    stride: u64,
    step_mode: VertexStepMode,
}

#[derive(Clone, Copy, Debug)]
pub enum VertexStepMode {
    Vertex,
    Instance,
}

impl VertexBuffer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        slot: u32,
        data: &[u8],
        attributes: Vec<VertexAttribute>,
        step_mode: VertexStepMode,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Vertex Buffer {}", slot)),
            size: data.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&buffer, 0, data);

        // Вычисляем stride (сумма размеров всех атрибутов)
        let stride = attributes.iter().map(|attr| attr.format.size()).sum();

        Self {
            buffer,
            slot,
            attributes,
            stride,
            step_mode,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, offset: u64, data: &[u8]) {
        queue.write_buffer(&self.buffer, offset, data);
    }

    pub fn slot(&self) -> u32 {
        self.slot
    }

    pub fn wgpu_layout(&self) -> wgpu::VertexBufferLayout<'static> {
        let attributes: Vec<_> = self
            .attributes
            .iter()
            .scan(0u64, |offset, attr| {
                let current_offset = *offset;
                *offset += attr.format.size();
                Some(wgpu::VertexAttribute {
                    offset: current_offset,
                    shader_location: attr.location,
                    format: attr.format.to_wgpu(),
                })
            })
            .collect();

        // ВАЖНО: Для static layout нужно вернуть owned данные
        // Это костыль, но WGPU требует 'static
        // В реальности нужно будет кешировать или использовать другой подход
        wgpu::VertexBufferLayout {
            array_stride: self.stride,
            step_mode: match self.step_mode {
                VertexStepMode::Vertex => wgpu::VertexStepMode::Vertex,
                VertexStepMode::Instance => wgpu::VertexStepMode::Instance,
            },
            attributes: Box::leak(attributes.into_boxed_slice()),
        }
    }
}

/// Формат индексов
#[derive(Clone, Copy, Debug)]
pub enum IndexFormat {
    Uint16,
    Uint32,
}

impl IndexFormat {
    pub fn to_wgpu(&self) -> wgpu::IndexFormat {
        match self {
            IndexFormat::Uint16 => wgpu::IndexFormat::Uint16,
            IndexFormat::Uint32 => wgpu::IndexFormat::Uint32,
        }
    }
}

/// Геометрия с множественными vertex буферами
pub struct Geometry {
    vertex_buffers: Vec<VertexBuffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_format: IndexFormat,

    topology: (),
    front_face: (),
    cull_mode: (),
    /// Количество элементов для рисования:
    /// - Если есть index_buffer: количество индексов
    /// - Если нет index_buffer: количество вершин
    element_count: u32,

    /// Количество инстансов для рисования
    instance_count: u32,
}

impl Geometry {
    /// Создать пустую геометрию
    pub fn new() -> Self {
        Self {
            vertex_buffers: vec![],
            index_buffer: None,
            index_format: IndexFormat::Uint32,
            element_count: 0,
            instance_count: 1,
            topology: (),
            front_face: (),
            cull_mode: (),
        }
    }

    /// Добавить vertex буфер
    pub fn add_vertex_buffer(&mut self, vertex_buffer: VertexBuffer) {
        self.vertex_buffers.push(vertex_buffer);
    }

    /// Установить index буфер
    pub fn set_index_buffer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        indices: &[u8],
        count: u32,
        format: IndexFormat,
    ) {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: indices.len() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&buffer, 0, indices);

        self.index_buffer = Some(buffer);
        self.index_format = format;
        self.element_count = count;
    }

    /// Установить количество вершин (для non-indexed геометрии)
    pub fn set_vertex_count(&mut self, count: u32) {
        if self.index_buffer.is_none() {
            self.element_count = count;
        }
    }

    /// Установить количество инстансов
    pub fn set_instance_count(&mut self, count: u32) {
        self.instance_count = count;
    }

    /// Получить количество элементов (индексов или вершин)
    pub fn element_count(&self) -> u32 {
        self.element_count
    }

    /// Получить количество инстансов
    pub fn instance_count(&self) -> u32 {
        self.instance_count
    }

    /// Есть ли index buffer
    pub fn is_indexed(&self) -> bool {
        self.index_buffer.is_some()
    }

    /// Получить все wgpu::VertexBufferLayout для pipeline
    pub fn vertex_buffer_layouts(&self) -> Vec<wgpu::VertexBufferLayout<'static>> {
        self.vertex_buffers
            .iter()
            .map(|vb| vb.wgpu_layout())
            .collect()
    }

    /// Установить все буферы в render pass
    pub fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for vb in &self.vertex_buffers {
            render_pass.set_vertex_buffer(vb.slot, vb.buffer.slice(..));
        }

        if let Some(ref index_buffer) = self.index_buffer {
            render_pass.set_index_buffer(index_buffer.slice(..), self.index_format.to_wgpu());
        }
    }

    /// Draw call с использованием сохраненных параметров
    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.index_buffer.is_some() {
            // Indexed draw: element_count = количество индексов
            render_pass.draw_indexed(0..self.element_count, 0, 0..self.instance_count);
        } else {
            // Non-indexed draw: element_count = количество вершин
            render_pass.draw(0..self.element_count, 0..self.instance_count);
        }
    }

    /// Draw call с кастомными параметрами
    pub fn draw_range<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        element_range: std::ops::Range<u32>,
        instances: std::ops::Range<u32>,
    ) {
        if self.index_buffer.is_some() {
            render_pass.draw_indexed(element_range, 0, instances);
        } else {
            render_pass.draw(element_range, instances);
        }
    }

    /// Получить ссылку на буфер по slot
    pub fn get_buffer(&self, slot: u32) -> Option<&VertexBuffer> {
        self.vertex_buffers.iter().find(|vb| vb.slot == slot)
    }

    /// Получить мутабельную ссылку на буфер по slot
    pub fn get_buffer_mut(&mut self, slot: u32) -> Option<&mut VertexBuffer> {
        self.vertex_buffers.iter_mut().find(|vb| vb.slot == slot)
    }

    /// Обновить index buffer
    pub fn update_index_buffer(&self, queue: &wgpu::Queue, offset: u64, data: &[u8]) {
        if let Some(ref buffer) = self.index_buffer {
            queue.write_buffer(buffer, offset, data);
        }
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper функции для создания геометрии
impl Geometry {
    /// Создать геометрию из одного interleaved буфера
    pub fn from_interleaved<V: bytemuck::Pod>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[V],
        attributes: Vec<VertexAttribute>,
        indices: Option<(&[u32], IndexFormat)>,
    ) -> Self {
        let mut geometry = Self::new();

        // Добавляем vertex buffer
        let vertex_buffer = VertexBuffer::new(
            device,
            queue,
            0,
            bytemuck::cast_slice(vertices),
            attributes,
            VertexStepMode::Vertex,
        );
        geometry.add_vertex_buffer(vertex_buffer);

        // Устанавливаем индексы или количество вершин
        if let Some((idx, format)) = indices {
            let index_data = match format {
                IndexFormat::Uint16 => {
                    let u16_indices: Vec<u16> = idx.iter().map(|&i| i as u16).collect();
                    bytemuck::cast_slice(&u16_indices).to_vec()
                }
                IndexFormat::Uint32 => bytemuck::cast_slice(idx).to_vec(),
            };
            geometry.set_index_buffer(device, queue, &index_data, idx.len() as u32, format);
        } else {
            geometry.set_vertex_count(vertices.len() as u32);
        }

        geometry
    }
}
