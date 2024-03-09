
struct Test;

impl rulf_3d::FrameworkLoop for Test {
	fn init(_device: &wgpu::Device, _queue: &wgpu::Queue, _surface_format: wgpu::TextureFormat) -> Self {
		Self
	}
}

impl rulf_3d::InputEvent for Test {

}


fn main(){
	rulf_3d::run::<Test>();
}