use ash::vk;

pub fn image_subresource_range(aspect_mask: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
  *vk::ImageSubresourceRange::builder()
    .aspect_mask(aspect_mask)
    .base_mip_level(0)
    .level_count(vk::REMAINING_MIP_LEVELS)
    .base_array_layer(0)
    .layer_count(vk::REMAINING_ARRAY_LAYERS)
}

pub fn image_create_info(extent: vk::Extent3D, format: vk::Format) -> vk::ImageCreateInfo {
  *vk::ImageCreateInfo::builder()
    .image_type(vk::ImageType::TYPE_2D)
    .format(format)
    .extent(extent)
    .mip_levels(1)
    .array_layers(1)
    .samples(vk::SampleCountFlags::TYPE_1)
    .tiling(vk::ImageTiling::OPTIMAL)
    .usage(
      vk::ImageUsageFlags::TRANSFER_SRC
        | vk::ImageUsageFlags::TRANSFER_DST
        | vk::ImageUsageFlags::STORAGE
        | vk::ImageUsageFlags::COLOR_ATTACHMENT,
    )
}

pub fn image_view_create_info(
  image: vk::Image,
  format: vk::Format,
  mask: vk::ImageAspectFlags,
) -> vk::ImageViewCreateInfo {
  let subresource = vk::ImageSubresourceRange::builder()
    .base_mip_level(0)
    .level_count(1)
    .base_array_layer(0)
    .layer_count(1)
    .aspect_mask(mask);

  *vk::ImageViewCreateInfo::builder()
    .view_type(vk::ImageViewType::TYPE_2D)
    .image(image)
    .format(format)
    .subresource_range(*subresource)
}
