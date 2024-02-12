use std::{
  cell::OnceCell,
  sync::{Arc, OnceLock},
};

use image::GenericImageView;
use wgpu::{Device, Extent3d, Queue, Texture};
use winit::window::Window;

use super::target::RenderTarget;

pub struct DiffuseTexture {
  pub texture: Texture,
  pub view: wgpu::TextureView,
  pub sampler: wgpu::Sampler,
  pub bind_group: wgpu::BindGroup,
}

impl DiffuseTexture {
  pub fn new(device: &Device, queue: &Queue, bytes: &[u8]) -> Self {
    let diffuse_image = image::load_from_memory(bytes).unwrap();
    let dimensions = diffuse_image.dimensions();
    let data = diffuse_image.to_rgba8();

    let size = wgpu::Extent3d {
      width: dimensions.0,
      height: dimensions.1,
      depth_or_array_layers: 1,
    };

    let unpadded_bytes_per_row = size.width * 4;
    let padded_bytes_per_row = ((unpadded_bytes_per_row + 255) & !255) as u32;

    let padded_data = {
      let mut padded_data = vec![0; (padded_bytes_per_row * size.height) as usize];

      for y in 0..size.height {
        let row_start = (y * padded_bytes_per_row) as usize;

        padded_data
          .get_mut(row_start..(row_start + unpadded_bytes_per_row as usize))
          .unwrap()
          .copy_from_slice(
            data
              .get((y * unpadded_bytes_per_row) as usize..((y + 1) * unpadded_bytes_per_row) as usize)
              .unwrap(),
          );
      }

      padded_data
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
      size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: RenderTarget::RENDER_TARGET_FORMAT,
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

    queue.write_texture(
      texture.as_image_copy(),
      &padded_data,
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(padded_bytes_per_row),
        rows_per_image: Some(size.height),
      },
      size,
    );

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
