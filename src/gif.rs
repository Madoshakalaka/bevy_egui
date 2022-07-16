use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::math::Size;
use bevy::reflect::TypeUuid;
use bevy::render::render_asset::{PrepareAssetError, RenderAsset};
use bevy::render::render_resource::{Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, SamplerDescriptor, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::{GpuImage, TextureFormatPixelInfo};
use image::Frame;

/// stores the data of each frame
#[derive(TypeUuid, Clone)]
#[uuid = "7ea27da8-6cf9-3ea1-9985-1d6bf6c16d7f"]
pub struct GifAnimation {
    /// notably stores the dimensions of the gif
    pub texture_descriptor: TextureDescriptor<'static>,
    /// the data of each frame
    pub frames: Vec<Frame>,
}

impl GifAnimation {
    /// `dimensions` is the width and the height
    pub fn new(dimensions: (u32, u32), frames: Vec<Frame>) -> Self {

        let texture_descriptor = TextureDescriptor {
            size: Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
            format: TextureFormat::Rgba8UnormSrgb,
            dimension: TextureDimension::D2,
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        };

        Self {
            frames,
            texture_descriptor,
        }
    }
}



impl RenderAsset for GifAnimation {
    type ExtractedAsset = GifAnimation;
    type PreparedAsset = Vec<GpuImage>;
    type Param = (SRes<RenderDevice>, SRes<RenderQueue>);

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    /// Converts the extracted image into a vec of [`GpuImage`]s.
    fn prepare_asset(
        animation: Self::ExtractedAsset,
        (render_device, render_queue): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        Ok(animation
            .frames
            .into_iter()
            .map(|f| {
                let texture = render_device.create_texture(&animation.texture_descriptor);
                let format_size = animation.texture_descriptor.format.pixel_size();
                // println!("one texture written");
                render_queue.write_texture(
                    ImageCopyTexture {
                        texture: &texture,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: TextureAspect::All,
                    },
                    f.buffer().as_raw(),
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(
                            std::num::NonZeroU32::new(
                                animation.texture_descriptor.size.width * format_size as u32,
                            )
                                .unwrap(),
                        ),
                        rows_per_image: if animation.texture_descriptor.size.depth_or_array_layers
                            > 1
                        {
                            std::num::NonZeroU32::new(animation.texture_descriptor.size.height)
                        } else {
                            None
                        },
                    },
                    animation.texture_descriptor.size,
                );

                let texture_view = texture.create_view(&TextureViewDescriptor::default());
                let size = Size::new(
                    animation.texture_descriptor.size.width as f32,
                    animation.texture_descriptor.size.height as f32,
                );
                let sampler = render_device.create_sampler(&SamplerDescriptor::default());
                GpuImage {
                    texture,
                    texture_view,
                    texture_format: animation.texture_descriptor.format,
                    sampler,
                    size,
                }
            })
            .collect())
    }
}
