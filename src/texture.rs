use image::GenericImageView;

pub struct Texture {
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new_depth_texture(container_width: u32, container_height: u32, device: &wgpu::Device) -> Self {
        let texture_size = wgpu::Extent3d {
            width: container_width,
            height: container_height,
            depth_or_array_layers: 1
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("depth_texture_view"),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("depth_texture_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            view: texture_view,
            sampler
        }
    }

    pub fn from_image_bytes(bytes: &[u8], name: &str, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let image = image::load_from_memory(bytes).unwrap();
        let (width, height) = image.dimensions();
        let image_rgba = image.to_rgba8();
    
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1
        };
    
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{name}_texture")),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[]
        });
    
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All
            },
            &image_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height)
            },
            texture_size
        );
    
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{name}_texture_view")),
            ..Default::default()
        });
    
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{name}_sampler")),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            view: texture_view,
            sampler
        }
    }

    pub fn from_rgba(name: &str, r: u8, g: u8, b: u8, a: u8, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let texture_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{name}_texture")),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[]
        });
    
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All
            },
            &[r, g, b, a],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1)
            },
            texture_size
        );
    
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{name}_texture_view")),
            ..Default::default()
        });
    
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{name}_sampler")),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            view: texture_view,
            sampler
        }
    }
}
