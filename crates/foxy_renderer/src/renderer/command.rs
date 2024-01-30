use foxy_utils::types::prelude::*;

pub enum Command {
  DrawPoint(Point3D),
  DrawLine(Point3D, Point3D),
  DrawTri(Point3D, Point3D, Point3D),
  DrawMesh(Vec<Point3D>),
}
