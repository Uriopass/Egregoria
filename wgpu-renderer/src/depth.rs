pub fn create_depth_texture(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
) -> (wgpu::Texture, wgpu::TextureView) {
    let desc = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        },
        mip_level_count: 1,
        array_layer_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: sc_desc.usage,
        label: None,
    };
    let depth_texture = device.create_texture(&desc);
    let depth_view = depth_texture.create_default_view();

    (depth_texture, depth_view)
}
