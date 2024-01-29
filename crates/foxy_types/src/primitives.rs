pub type Point2D = glam::Vec2;
pub type Point3D = glam::Vec3;

pub type Line2D = (Point2D, Point2D);
pub type Line3D = (Point3D, Point3D);

pub type Matrix3D = glam::Mat3;
pub type Matrix4D = glam::Mat4;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dimensions {
  pub width: i32,
  pub height: i32,
}
