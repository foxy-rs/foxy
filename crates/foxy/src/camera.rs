use nalgebra::{Matrix4, Perspective3, Point3, Vector3};

pub struct Camera {
  eye: Point3<f32>,
  target: Point3<f32>,
  up: Vector3<f32>,
  aspect: f32,
  fov_y: f32,
  z_near: f32,
  z_far: f32,
}

impl Camera {
  #[rustfmt::skip]
  pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
  );

  fn build_view_proj_matrix(&self) -> Matrix4<f32> {
    let view = Matrix4::face_towards(&self.eye, &self.target, &self.up);
    let proj = Perspective3::new(self.aspect, self.fov_y, self.z_near, self.z_far);
    Self::OPENGL_TO_WGPU_MATRIX * proj.as_matrix() * view
  }
}
