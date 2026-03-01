use glam::{Mat3, Mat4, Quat, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    position: Vec3,
    orientation: Quat,
    scale: Vec3,
    model: Mat4,
    normal_matrix: Mat3,
}

impl Transform {
    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn orientation(&self) -> Quat {
        self.orientation
    }

    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.invalidate_model();
    }

    pub fn set_orientation(&mut self, orientation: Quat) {
        self.orientation = orientation;
        self.invalidate_model();
        self.invalidate_normal_matrix();
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.invalidate_model();
        self.invalidate_normal_matrix();
    }

    fn is_invalid_model(&self) -> bool {
        self.model.w_axis.w == 0.0
    }

    fn is_invalid_normal_matrix(&self) -> bool {
        self.normal_matrix.x_axis.y.is_nan()
    }

    fn invalidate_model(&mut self) {
        self.model = Self::invalid_model();
    }

    fn invalidate_normal_matrix(&mut self) {
        self.normal_matrix = Self::invalid_normal_matrix();
    }

    pub fn get_model(&mut self) -> Mat4 {
        if self.is_invalid_model() {
            // Recompute model matrix: TRS (Translation * Rotation * Scale)
            let translation = Mat4::from_translation(self.position);
            let rotation = Mat4::from_quat(self.orientation);
            let scale = Mat4::from_scale(self.scale);
            self.model = translation * rotation * scale;
        }
        self.model
    }

    pub fn get_normal_matrix(&mut self) -> Mat3 {
        if self.is_invalid_normal_matrix() {
            // Recompute normal matrix: transpose(inverse(mat3(model)))
            // This correctly handles non-uniform scale
            let model = self.get_model();
            let model_3x3 = Mat3::from_mat4(model);
            self.normal_matrix = model_3x3.inverse().transpose();
        }
        self.normal_matrix
    }

    fn invalid_model() -> Mat4 {
        Mat4::ZERO
    }

    fn invalid_normal_matrix() -> Mat3 {
        let mut matrix = Mat3::ZERO;
        matrix.x_axis.y = f32::NAN; // NaN гарантирует что это невалидная матрица
        matrix
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            scale: Vec3::ONE,
            model: Self::invalid_model(),
            normal_matrix: Self::invalid_normal_matrix(),
        }
    }
}
