use image::GenericImageView;
use wgpu::{Device, Extent3d, Queue, Texture};

pub struct DiffuseTexture {
  pub texture: Texture,
  pub view: wgpu::TextureView,
  pub sampler: wgpu::Sampler,
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

    let texture = device.create_texture(&wgpu::TextureDescriptor {
      size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8UnormSrgb,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      label: Some("Diffuse::texture"),
      view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("Diffuse::sampler"),
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
      &data,
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(4 * size.width),
        rows_per_image: Some(size.height),
      },
      size,
    );

    Self {
      texture,
      view,
      sampler,
    }
  }
}
