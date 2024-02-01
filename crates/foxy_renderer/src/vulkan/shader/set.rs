use std::sync::RwLockReadGuard;

use foxy_utils::types::handle::Handle;

use super::{
  stage::{compute::Compute, fragment::Fragment, geometry::Geometry, mesh::Mesh, vertex::Vertex},
  storage::ShaderStore,
  Shader,
};

#[derive(Clone)]
pub struct NoVertex;
#[derive(Clone)]
pub struct HasVertex(Handle<Shader<Vertex>>);

#[derive(Clone)]
pub struct NoFragment;
#[derive(Clone)]
pub struct HasFragment(Handle<Shader<Fragment>>);

#[derive(Clone)]
pub struct NoCompute;
#[derive(Clone)]
pub struct HasCompute(Handle<Shader<Compute>>);

#[derive(Clone)]
pub struct NoGeometry;
#[derive(Clone)]
pub struct HasGeometry(Handle<Shader<Geometry>>);

#[derive(Clone)]
pub struct NoMesh;
#[derive(Clone)]
pub struct HasMesh(Handle<Shader<Mesh>>);

#[derive(Clone)]
pub struct ShaderSet<V, F, C, G, M> {
  store: Handle<ShaderStore>,
  vertex: V,
  fragment: F,
  compute: C,
  geometry: G,
  mesh: M,
}

impl ShaderSet<NoVertex, NoFragment, NoCompute, NoGeometry, NoMesh> {
  pub fn new(store: Handle<ShaderStore>) -> Self {
    Self {
      store,
      vertex: NoVertex,
      fragment: NoFragment,
      compute: NoCompute,
      geometry: NoGeometry,
      mesh: NoMesh,
    }
  }
}

impl<F, C, G, M> ShaderSet<NoVertex, F, C, G, M> {
  pub fn with_vertex(mut self, vertex: &'static str) -> ShaderSet<HasVertex, F, C, G, M> {
    let vertex = HasVertex(self.store.get_mut().get_vertex(vertex));
    ShaderSet {
      store: self.store,
      vertex,
      fragment: self.fragment,
      compute: self.compute,
      geometry: self.geometry,
      mesh: self.mesh,
    }
  }
}

impl<F, C, G, M> ShaderSet<HasVertex, F, C, G, M> {
  pub fn vertex(&self) -> RwLockReadGuard<'_, Shader<Vertex>> {
    self.vertex.0.get()
  }
}

impl<V, C, G, M> ShaderSet<V, NoFragment, C, G, M> {
  pub fn with_fragment(mut self, fragment: &'static str) -> ShaderSet<V, HasFragment, C, G, M> {
    let fragment = HasFragment(self.store.get_mut().get_fragment(fragment));
    ShaderSet {
      store: self.store,
      vertex: self.vertex,
      fragment,
      compute: self.compute,
      geometry: self.geometry,
      mesh: self.mesh,
    }
  }
}

impl<V, C, G, M> ShaderSet<V, HasFragment, C, G, M> {
  pub fn fragment(&self) -> RwLockReadGuard<'_, Shader<Fragment>> {
    self.fragment.0.get()
  }
}

impl<V, F, G, M> ShaderSet<V, F, NoCompute, G, M> {
  pub fn with_compute(mut self, compute: &'static str) -> ShaderSet<V, F, HasCompute, G, M> {
    let compute = HasCompute(self.store.get_mut().get_compute(compute));
    ShaderSet {
      store: self.store,
      vertex: self.vertex,
      fragment: self.fragment,
      compute,
      geometry: self.geometry,
      mesh: self.mesh,
    }
  }
}

impl<V, F, G, M> ShaderSet<V, F, HasCompute, G, M> {
  pub fn compute(&self) -> RwLockReadGuard<'_, Shader<Compute>> {
    self.compute.0.get()
  }
}

impl<V, F, C, M> ShaderSet<V, F, C, NoGeometry, M> {
  pub fn with_geometry(mut self, geometry: &'static str) -> ShaderSet<V, F, C, HasGeometry, M> {
    let geometry = HasGeometry(self.store.get_mut().get_geometry(geometry));
    ShaderSet {
      store: self.store,
      vertex: self.vertex,
      fragment: self.fragment,
      compute: self.compute,
      geometry,
      mesh: self.mesh,
    }
  }
}

impl<V, F, C, M> ShaderSet<V, F, C, HasGeometry, M> {
  pub fn geometry(&self) -> RwLockReadGuard<'_, Shader<Geometry>> {
    self.geometry.0.get()
  }
}

impl<V, F, C, G> ShaderSet<V, F, C, G, NoMesh> {
  pub fn with_mesh(mut self, mesh: &'static str) -> ShaderSet<V, F, C, G, HasMesh> {
    let mesh = HasMesh(self.store.get_mut().get_mesh(mesh));
    ShaderSet {
      store: self.store,
      vertex: self.vertex,
      fragment: self.fragment,
      compute: self.compute,
      geometry: self.geometry,
      mesh,
    }
  }
}

impl<V, F, C, G> ShaderSet<V, F, C, G, HasMesh> {
  pub fn mesh(&self) -> RwLockReadGuard<'_, Shader<Mesh>> {
    self.mesh.0.get()
  }
}
