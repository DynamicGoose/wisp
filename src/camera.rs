use glam::{Mat4, Vec3};

/*
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
]);
*/

#[derive(Clone, Copy)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub viewport: Option<Viewport>,
}

impl Camera {
    pub fn build_view_projection_matrix(&self, aspect: f32) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = if self.viewport.is_some() {
            Mat4::perspective_rh(
                self.fovy,
                self.viewport.as_ref().unwrap().w / self.viewport.as_ref().unwrap().h,
                self.znear,
                self.zfar,
            )
        } else {
            Mat4::perspective_rh(self.fovy, aspect, self.znear, self.zfar)
        };

        proj * view
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_projection: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_projection: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_projection(&mut self, camera: &Camera, aspect: f32) {
        // We're using Vector4 because of the uniforms 16 byte spacing requirement
        self.view_position = [camera.eye.x, camera.eye.y, camera.eye.z, 1.0];
        self.view_projection = (camera.build_view_projection_matrix(aspect)).to_cols_array_2d();
    }
}

#[derive(Clone, Copy)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
