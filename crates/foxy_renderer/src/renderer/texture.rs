use std::{mem::size_of, path::PathBuf, sync::OnceLock};

use image::{EncodableLayout, GenericImageView, Pixel};
use itertools::Itertools;
use wgpu::{Device, Queue, Texture};

use super::Renderer;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub PathBuf);

impl From<&str> for TextureHandle {
  fn from(value: &str) -> Self {
    Self(value.into())
  }
}

pub struct DiffuseTexture {
  pub texture: Texture,
  pub view: wgpu::TextureView,
  pub sampler: wgpu::Sampler,
  pub bind_group: wgpu::BindGroup,
}

impl DiffuseTexture {
  pub fn new(
    device: &Device,
    queue: &Queue,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    size: wgpu::Extent3d,
  ) -> Self {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
      size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format,
      usage,
      label: Some("Diffuse Texture"),
      view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("Diffuse Sampler"),
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Nearest,
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
    }
  }

  pub fn from_bytes(device: &Device, queue: &Queue, bytes: &[u8]) -> Self {
    let diffuse_image = image::load_from_memory(bytes).unwrap();
    let dimensions = diffuse_image.dimensions();
    let data = diffuse_image.to_rgba8();

    let size = wgpu::Extent3d {
      width: dimensions.0,
      height: dimensions.1,
      depth_or_array_layers: 1,
    };

    let unpadded_bytes_per_row = size_of::<[u8; 4]>() * size.width as usize;
    let padding = (256 - (unpadded_bytes_per_row % 256)) % 256;
    let mut padded_data = Vec::with_capacity((unpadded_bytes_per_row + padding) * size.height as usize);
    for (_, row) in data.enumerate_rows() {
      padded_data.extend_from_slice(
        &row
          .flat_map(|(_, _, value)| {
            let bytes = value.channels();
            bytes.as_bytes().to_vec()
          })
          .collect_vec(),
      );
      padded_data.resize(padded_data.len() + padding, 0);
    }

    let texture = Self::new(
      device,
      queue,
      Renderer::SURFACE_FORMAT,
      wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      size,
    );

    queue.write_texture(
      texture.texture.as_image_copy(),
      &padded_data,
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some((unpadded_bytes_per_row + padding) as u32),
        rows_per_image: Some(size.height),
      },
      size,
    );

    texture
  }

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
