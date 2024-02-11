use std::num::NonZeroU32;

use wgpu::{util::*, *};

use crate::Image;

impl<T, const N: usize> Image<T, N> {
    /// Get the size as a [`wgpu::Extend3d`].
    pub fn wgpu_size(&self) -> Extent3d {
        Extent3d {
            width: self.width(),
            height: self.height(),
            depth_or_array_layers: 1,
        }
    }
}

impl<T: AsRef<[u8]>> Image<T, 4> {
    /// Upload this image to the gpu, returning a [`wgpu::Texture`].
    pub fn send(&self, dev: &Device, q: &Queue, usage: TextureUsages) -> Texture {
        dev.create_texture_with_data(
            &q,
            &TextureDescriptor {
                label: None,
                size: self.wgpu_size(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                view_formats: &[],
                usage,
            },
            util::TextureDataOrder::LayerMajor,
            self.bytes(),
        )
    }
}

impl Image<Box<[u8]>, 4> {
    /// Downloads a purportedly [`TextureFormat::Rgba8Unorm`] image from the gpu.
    /// # Panics
    ///
    /// When a "error occurs while trying to async map a buffer".
    pub fn download(
        dev: &Device,
        q: &Queue,
        texture: &Texture,
        (width, height): (NonZeroU32, NonZeroU32),
    ) -> Self {
        let mut encoder = dev.create_command_encoder(&CommandEncoderDescriptor { label: None });
        let texture_size = Extent3d {
            width: width.get(),
            height: height.get(),
            depth_or_array_layers: 1,
        };

        let row = width.get() as usize * 4;
        let pad = {
            let padding = (256 - row % 256) % 256;
            row + padding
        };

        let output_buffer = dev.create_buffer(&BufferDescriptor {
            label: None,
            size: pad as u64 * height.get() as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(pad as u32),
                    rows_per_image: Some(height.get()),
                },
            },
            texture_size,
        );
        q.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, Result::unwrap);

        dev.poll(wgpu::Maintain::Wait);

        let mut out = crate::uninit::Image::<_, 4>::new(width, height);
        for (padded, pixels) in buffer_slice
            .get_mapped_range()
            .chunks_exact(pad)
            .zip(out.buf().chunks_exact_mut(row))
        {
            ::core::mem::MaybeUninit::write_slice(pixels, &padded[..row]);
        }

        unsafe { out.assume_init().boxed() }
    }
}
