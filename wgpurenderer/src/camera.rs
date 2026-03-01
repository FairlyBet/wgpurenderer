use glam::{Mat4, Quat, Vec3};
use crate::transform::Transform;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    projection: Mat4,
    view_projection: Mat4,
    cached_position: Vec3,
    cached_orientation: Quat,
}

impl Camera {
    pub fn set_projection(&mut self, projection: Mat4) {
        self.projection = projection;
        self.invalidate_view_projection();
    }

    pub fn set_perspective(&mut self, fov: f32, aspect: f32, near: f32, far: f32) {
        self.projection = Mat4::perspective_rh(fov, aspect, near, far);
        self.invalidate_view_projection();
    }

    pub fn get_view_projection(&mut self, transform: &Transform) -> Mat4 {
        if self.is_invalid_view_projection() || self.is_transform_changed(transform) {
            let position = transform.position();
            let orientation = transform.orientation();

            // Compute view matrix: projection * rotation(inverse(orientation)) * translate(-position)
            let rotation = Mat4::from_quat(orientation.inverse());
            let translation = Mat4::from_translation(-position);
            let view = rotation * translation;
            self.view_projection = self.projection * view;

            // Update cached values
            self.cached_position = position;
            self.cached_orientation = orientation;
        }
        self.view_projection
    }

    fn is_invalid_view_projection(&self) -> bool {
        self.view_projection.w_axis.w == 0.0
    }

    fn is_transform_changed(&self, transform: &Transform) -> bool {
        transform.position() != self.cached_position
            || transform.orientation() != self.cached_orientation
    }

    fn invalidate_view_projection(&mut self) {
        self.view_projection = Self::invalid_view_projection();
    }

    fn invalid_view_projection() -> Mat4 {
        Mat4::ZERO
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            projection: Mat4::IDENTITY,
            view_projection: Self::invalid_view_projection(),
            cached_position: Vec3::ZERO,
            cached_orientation: Quat::IDENTITY,
        }
    }
}
