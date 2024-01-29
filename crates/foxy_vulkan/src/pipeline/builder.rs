use std::path::PathBuf;

use foxy_types::handle::Handle;

use super::{
  config::{HasLayout, HasRenderPass, RenderPipelineConfig},
  RenderPipeline,
};
use crate::{
  device::Device,
  error::VulkanError,
  shader::{
    stage::{fragment::Fragment, vertex::Vertex},
    Shader,
  },
};

pub struct VertexShaderMissing;
pub struct VertexShaderSpecified(Handle<Shader<Vertex>>);
pub struct FragmentShaderMissing;
pub struct FragmentShaderSpecified(Handle<Shader<Fragment>>);

pub struct ConfigMissing;
pub struct ConfigSpecified(RenderPipelineConfig<HasLayout, HasRenderPass>);

pub struct RenderPipelineBuilder<VS, FS, PC> {
  vulkan: Handle<Device>,
  vertex_shader: VS,
  fragment_shader: FS,
  config: PC,
}

impl RenderPipelineBuilder<VertexShaderMissing, FragmentShaderMissing, ConfigMissing> {
  pub fn new(vulkan: Handle<Device>) -> Self {
    Self {
      vulkan,
      vertex_shader: VertexShaderMissing,
      fragment_shader: FragmentShaderMissing,
      config: ConfigMissing,
    }
  }
}

impl<FS, PC> RenderPipelineBuilder<VertexShaderMissing, FS, PC> {
  pub fn with_vertex_shader<P: Into<PathBuf>>(
    mut self,
    path: P,
  ) -> RenderPipelineBuilder<VertexShaderSpecified, FS, PC> {
    RenderPipelineBuilder {
      vulkan: self.vulkan.clone(),
      vertex_shader: VertexShaderSpecified(self.vulkan.get_mut().shaders().get_vertex(path)),
      fragment_shader: self.fragment_shader,
      config: self.config,
    }
  }
}

impl<VS, PC> RenderPipelineBuilder<VS, FragmentShaderMissing, PC> {
  pub fn with_fragment_shader<P: Into<PathBuf>>(
    mut self,
    path: P,
  ) -> RenderPipelineBuilder<VS, FragmentShaderSpecified, PC> {
    RenderPipelineBuilder {
      vulkan: self.vulkan.clone(),
      vertex_shader: self.vertex_shader,
      fragment_shader: FragmentShaderSpecified(self.vulkan.get_mut().shaders().get_fragment(path)),
      config: self.config,
    }
  }
}

impl<VS, FS> RenderPipelineBuilder<VS, FS, ConfigMissing> {
  pub fn with_config(
    self,
    config: RenderPipelineConfig<HasLayout, HasRenderPass>,
  ) -> RenderPipelineBuilder<VS, FS, ConfigSpecified> {
    RenderPipelineBuilder {
      vulkan: self.vulkan,
      vertex_shader: self.vertex_shader,
      fragment_shader: self.fragment_shader,
      config: ConfigSpecified(config),
    }
  }
}

impl RenderPipelineBuilder<VertexShaderSpecified, FragmentShaderSpecified, ConfigSpecified> {
  pub fn build(self) -> Result<RenderPipeline, VulkanError> {
    Ok(RenderPipeline::new(
      self.vulkan,
      self.config.0,
      self.vertex_shader.0,
      self.fragment_shader.0,
    )?)
  }
}
