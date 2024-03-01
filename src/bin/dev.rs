struct TestGameLoop(bool, bool);

impl rulf_3d::GameLoop for TestGameLoop
{
    fn startup(&mut self, device: &wgpu::Device)
	{
		println!("my test startup");
	}
    fn process(&mut self, delta: f64)
	{
		if self.0 == false
		{
			self.0 = true;
			println!("my test process");
		}
	}
    fn render(&mut self, device: &wgpu::Device, surface: &wgpu::Surface, queue: &wgpu::Queue)
	{
		if self.1 == false
		{
			self.1 = true;
			println!("my test render");
		}
	}
}

fn main()
{
	rulf_3d::run_dev(TestGameLoop(false, false));
}