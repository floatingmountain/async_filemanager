pub trait Material {
    fn create_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup;
}
