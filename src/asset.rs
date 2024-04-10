use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::collections::HashMap;
 
pub struct ShaderSource;
 
impl ShaderSource {
    pub const FILLSCREEN: &'static str = include_str!("asset/fillscreen.wgsl");
    pub const FIRSTPERSON_WALL_COMPUTE: &'static str = include_str!("asset/firstperson_wall_compute.wgsl");
    pub const FIRSTPERSON_WALL_FRAG: &'static str = include_str!("asset/firstperson_wall_frag.wgsl");
    pub const FIRSTPERSON_FLOORCEIL: &'static str = include_str!("asset/firstperson_floorceil.wgsl");
    pub const MINIMAP_ACTOR: &'static str = include_str!("asset/minimap_actor.wgsl");
    pub const MINIMAP_WALL: &'static str = include_str!("asset/minimap_wall.wgsl");
}
 
pub struct AssetServer { //TODO: AssetServer
    shaders: HashMap<&'static str, wgpu::ShaderModule>,
    images: HashMap<&'static str, image::DynamicImage>,
	textures: HashMap<&'static str, wgpu::Texture>
}

pub enum ArrayOrder {
	Column, Row
}

pub enum TextureType {
	Full,
	Partial {
		x: u32,
		y: u32,
		width: u32,
		height: u32
	},
	Grid {
		order: ArrayOrder,
		x: u32,
		y: u32
	},
	PaddingGrid {
		order: ArrayOrder,
		x: u32,
		y: u32,
		left: u32,
		right: u32,
		top: u32,
		bottom: u32
	}
}
 
pub enum AssetServerError {
    DuplicatedName, 
	OpenFileFailed(std::io::Error), 
	ReadImageFailed(image::ImageError), 
	NameNotFound, 
	InvalidSize, 
	InvalidPosition
}

impl AssetServer {
	pub fn create_test_asset_server(device: &wgpu::Device, queue: &wgpu::Queue) -> AssetServer {
		let mut asset_server = AssetServer::new();

		if let Err(_) = asset_server.load_image("all_6", "src/asset/all_6.jpg") {
			panic!("Failed to load image src/asset/all_6.jpg");
		}

		if let Err(_) = asset_server.create_image_texture(
			device, queue, "all_6", 
			&TextureType::Grid {
				order: ArrayOrder::Row,
				x: 5, y: 5
		}) {
			panic!("Failed to create texture all_6");
		}

		asset_server
	}
}
 
impl AssetServer {
    pub fn new() -> AssetServer {
        AssetServer {
            shaders: HashMap::new(),
            images: HashMap::new(),
			textures: HashMap::new()
        }
    }
    fn load_str(path: &Path) -> io::Result<String> {
        let mut file = match File::open(path) {
            Err(e) => return Err(e),
            Ok(f) => f
        };
 
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(e) => return Err(e),
            Ok(_) => return Ok(s)
        }
    }
 
    fn load_byte(path: &Path) -> io::Result<Vec<u8>> {
        let mut file = match File::open(path) {
            Err(e) => return Err(e),
            Ok(f) => f
        };
        let mut b = Vec::<u8>::new();
        match file.read_to_end(&mut b) {
            Err(e) => return Err(e),
            Ok(_) => return Ok(b)
        }
    }
 
    pub fn load_wgsl(&mut self, device: &wgpu::Device, name: &'static str, path: &str) -> Result<(), AssetServerError> {
        if self.shaders.contains_key(name) {
            return Err(AssetServerError::DuplicatedName);
        }
 
        let string = match Self::load_str(Path::new(path)) {
            Ok(s) => s,
            Err(e) => return Err(AssetServerError::OpenFileFailed(e))
        };
 
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::Wgsl(string.as_str().into())
        });
 
        self.shaders.insert(name, shader);
 
        Ok(())
    }
 
    pub fn load_image(&mut self, name: &'static str, path: &str) -> Result<(), AssetServerError> {
        if self.images.contains_key(name) {
            return Err(AssetServerError::DuplicatedName);
        }
 
        let bytes = match Self::load_byte(Path::new(path)) {
            Ok(b) => b,
            Err(e) => return Err(AssetServerError::OpenFileFailed(e))
        };
 
        let image = match image::load_from_memory(bytes.as_slice()) {
            Ok(img) => img,
            Err(e) => return Err(AssetServerError::ReadImageFailed(e))
        };
 
        self.images.insert(name, image);
 
        Ok(())
    }

	pub fn get_texture(&self, name: &'static str) -> Option<&wgpu::Texture> {
		self.textures.get(name)
	}

	pub fn create_image_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, name: &'static str, textype: &TextureType) -> Result<(), AssetServerError> {
		if self.textures.contains_key(name) {
			return Err(AssetServerError::DuplicatedName);
		}

		let image = match self.images.get(name) {
			None => return Err(AssetServerError::NameNotFound),
			Some(img) => img
		};
		let data = image.to_rgba8();

		match textype {
			TextureType::Full => { // NOTE: Not Tested
				let size = wgpu::Extent3d {
					width: image.width(),
					height: image.height(),
					depth_or_array_layers: 1
				};
				let texture = device.create_texture(&wgpu::TextureDescriptor {
					label: Some(name),
					size,
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
						origin: wgpu::Origin3d {
							x:0, y:0, z:0
						},
						aspect: wgpu::TextureAspect::All
					}, 
					&data,
					wgpu::ImageDataLayout {
						offset: 0,
						bytes_per_row: Some(size.width * 4),
						rows_per_image: Some(size.height)
					},
					size
				);
				self.textures.insert(name, texture);
				Ok(())
			},
			TextureType::Grid { order, x, y } => {
				let length = x * y;
				if length == 0 {
					return Err(AssetServerError::InvalidSize);
				}
				let size = wgpu::Extent3d {
					width: image.width() / x,
					height: image.height() / y,
					depth_or_array_layers: length
				};
				let texture = device.create_texture(&wgpu::TextureDescriptor {
					label: Some(name),
					size,
					mip_level_count: 1,
					sample_count: 1,
					dimension: wgpu::TextureDimension::D2,
					format: wgpu::TextureFormat::Rgba8UnormSrgb,
					usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
					view_formats: &[]
				});

				let offset = |i| match order {
					ArrayOrder::Row => (size.width * (i % x) + size.width * x * size.height * (i / x)) * 4,
					ArrayOrder::Column => (size.width * x * size.height * (i % x) + size.width * (i / x)) * 4
				};

				for l in 0..length {
					queue.write_texture(
						wgpu::ImageCopyTexture {
							texture: &texture,
							aspect: wgpu::TextureAspect::All,
							mip_level: 0,
							origin: wgpu::Origin3d {
								x: 0, y: 0, z: l
							}
						},
						&data,
						wgpu::ImageDataLayout {
							offset: offset(l) as u64,
							bytes_per_row: Some(4 * size.width * x),
							rows_per_image: Some(size.height)
						},
						wgpu::Extent3d {
							width: size.width,
							height: size.height,
							depth_or_array_layers: 1
						}
					)
				};

				self.textures.insert(name, texture);
				Ok(())
			},
			TextureType::Partial { x, y, width, height } => {
				todo!()
			},
			TextureType::PaddingGrid { order, x, y, left, right, top, bottom } => {
				todo!()
			}
		}
	}
}
 
#[test]
fn asset_server_functionality() {
    use crate::webgpu::MinimalWebGPU;
	use crate::webgpu::WebGPUDevice;
    let webgpu = MinimalWebGPU::_new();
	let (device, _) = webgpu.get_device();
    let mut asset_server =  AssetServer::new();
 
    assert!(asset_server.load_image("testimg", "src/asset/all_6.jpg").is_ok());
    assert!(asset_server.load_wgsl(device, "test", "src/asset/fillscreen.wgsl").is_ok());
}