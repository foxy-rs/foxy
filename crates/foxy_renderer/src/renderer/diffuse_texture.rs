use std::{mem::size_of, path::PathBuf, sync::OnceLock};

use image::{EncodableLayout, GenericImageView, ImageBuffer, Pixel, Rgba};
use itertools::Itertools;
use wgpu::{Device, Queue, Texture};

use super::Renderer;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextureHandle {
  FromFile(PathBuf),
  New(String),
}

impl TextureHandle {
  pub fn as_path_buf(&self) -> PathBuf {
    match self {
      TextureHandle::FromFile(x) => x.clone(),
      TextureHandle::New(x) => x.into(),
    }
  }

  pub fn is_file(&self) -> bool {
    match self {
      TextureHandle::FromFile(_) => true,
      TextureHandle::New(_) => false,
    }
  }
}
// impl From<&str> for TextureHandle {
//   fn from(value: &str) -> Self {
//     Self(value.into())
//   }
// }

pub struct DiffuseTexture {
  pub texture: Texture,
  pub view: wgpu::TextureView,
  pub sampler: wgpu::Sampler,
  pub bind_group: wgpu::BindGroup,
  pub size: wgpu::Extent3d,
}

impl DiffuseTexture {
  pub fn new(device: &Device, size: wgpu::Extent3d) -> Self {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
      size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: Renderer::SURFACE_FORMAT,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      label: Some("Diffuse Texture"),
      view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("Diffuse Sampler"),
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Nearest,
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Diffuse Bind Group"),
      layout: Self::bind_group_layout(device),
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(&view),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(&sampler),
        },
      ],
    });

    Self {
      texture,
      view,
      sampler,
      bind_group,
      size,
    }
  }

  pub fn resize(&mut self, device: &Device, size: wgpu::Extent3d) {
    *self = Self::new(device, size);
  }

  pub fn write_image(&self, queue: &Queue, data: ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let unpadded_bytes_per_row = size_of::<[u8; 4]>() * self.size.width as usize;
    let padding = (256 - (unpadded_bytes_per_row % 256)) % 256;
    let mut padded_data = Vec::with_capacity((unpadded_bytes_per_row + padding) * self.size.height as usize);
    for (_, row) in data.enumerate_rows() {
      padded_data.extend_from_slice(
        &row
          .flat_map(|(_, _, value)| value.channels().as_bytes().to_vec())
          .collect_vec(),
      );
      padded_data.resize(padded_data.len() + padding, 0);
    }

    queue.write_texture(
      self.texture.as_image_copy(),
      &padded_data,
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some((unpadded_bytes_per_row + padding) as u32),
        rows_per_image: Some(self.size.height),
      },
      self.size,
    );
  }

  // pub fn write_rgba8(&self, queue: &Queue, data: &[[f32; 4]]) {
  //   let data = data.iter().flat_map(|p| Rgba::from_slice(p)).collect();

  //   let unpadded_bytes_per_row = size_of::<[u8; 4]>() * self.size.width as
  // usize;   let padding = (256 - (unpadded_bytes_per_row % 256)) % 256;
  //   let mut padded_data = Vec::with_capacity((unpadded_bytes_per_row + padding)
  // * self.size.height as usize);   for (_, row) in data.enumerate_rows() {
  //   padded_data.extend_from_slice( &row .flat_map(|(_, _, value)| { let bytes =
  //   value.channels(); bytes.as_bytes().to_vec() }) .collect_vec(), );
  //   padded_data.resize(padded_data.len() + padding, 0); }

  //   queue.write_texture(
  //     self.texture.as_image_copy(),
  //     &padded_data,
  //     wgpu::ImageDataLayout {
  //       offset: 0,
  //       bytes_per_row: Some((unpadded_bytes_per_row + padding) as u32),
  //       rows_per_image: Some(self.size.height),
  //     },
  //     self.size,
  //   );
  // }

  pub fn bind_group_layout(device: &Device) -> &wgpu::BindGroupLayout {
    static BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();

    BIND_GROUP_LAYOUT.get_or_init(|| {
      device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Diffuse Bind Group Layout"),
        entries: &[
          wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
              multisampled: false,
              view_dimension: wgpu::TextureViewDimension::D2,
              sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
          },
          wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            // This should match the filterable field of the
            // corresponding Texture entry above.
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
          },
        ],
      })
    })
  }
}
