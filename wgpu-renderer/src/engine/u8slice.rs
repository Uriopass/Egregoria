/// Trait is needed to use bytemuck's conversion on external types
pub trait ToU8Slice {
    fn to_slice(&self) -> &[u8];
}

macro_rules! u8slice_impl {
    ($t: ty) => {
        unsafe impl bytemuck::Pod for $t {}
        unsafe impl bytemuck::Zeroable for $t {}
        impl crate::engine::ToU8Slice for [$t] {
            fn to_slice(&self) -> &[u8] {
                bytemuck::cast_slice(self)
            }
        }
    };
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct Matrix4NT(cgmath::Matrix4<f32>);

unsafe impl bytemuck::Pod for Matrix4NT {}
unsafe impl bytemuck::Zeroable for Matrix4NT {}

impl ToU8Slice for [cgmath::Matrix4<f32>] {
    fn to_slice(&self) -> &[u8] {
        let v: &[Matrix4NT] = unsafe { std::mem::transmute(self) };
        bytemuck::cast_slice(v)
    }
}
