pub struct Display {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}

impl Display {
    pub fn format(&self) -> &wgpu::TextureFormat {
        &self.config.format
    }

    pub fn width(&self) -> u32 {
        self.config.width
    }

    pub fn height(&self) -> u32 {
        self.config.height
    }

    pub fn new(
        surface: wgpu::Surface,
        config: wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
    ) -> Self {
        surface.configure(&device, &config);

        Display { surface, config }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(device, &self.config);
    }

    pub fn texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        Ok(output)
    }
}
