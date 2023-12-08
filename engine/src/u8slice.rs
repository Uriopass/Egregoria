use geom::{Matrix4, Vec3, Vec4};

/// Trait is needed to use bytemuck's conversion on external types
pub trait ToU8Slice: Sized {
    fn cast_slice(self_slice: &[Self]) -> &[u8];
}

#[macro_export]
macro_rules! u8slice_impl {
    ($t: ty) => {
        unsafe impl bytemuck::Pod for $t {}
        unsafe impl bytemuck::Zeroable for $t {}
        impl $crate::ToU8Slice for $t {
            fn cast_slice(self_slice: &[Self]) -> &[u8] {
                bytemuck::cast_slice(self_slice)
            }
        }
    };
}

#[repr(transparent)]
#[derive(Copy, Clone)]
struct NewType<T>(T);

unsafe impl<T> bytemuck::Zeroable for NewType<T> {}
unsafe impl<T: Copy + 'static> bytemuck::Pod for NewType<T> {}

#[macro_export]
macro_rules! u8slice_impl_external {
    ($t: ty) => {
        impl ToU8Slice for $t {
            fn cast_slice<'a>(self_slice: &'a [Self]) -> &'a [u8] {
                let v: &'a [NewType<$t>] =
                    unsafe { &*(self_slice as *const [$t] as *const [NewType<$t>]) };
                bytemuck::cast_slice(v)
            }
        }
    };
}

u8slice_impl_external!(Matrix4);
u8slice_impl_external!(Vec3);
u8slice_impl_external!(Vec4);

impl ToU8Slice for f32 {
    fn cast_slice(self_slice: &[f32]) -> &[u8] {
        bytemuck::cast_slice(self_slice)
    }
}

impl ToU8Slice for u32 {
    fn cast_slice(self_slice: &[u32]) -> &[u8] {
        bytemuck::cast_slice(self_slice)
    }
}

impl ToU8Slice for [f32; 4] {
    fn cast_slice(self_slice: &[[f32; 4]]) -> &[u8] {
        bytemuck::cast_slice(self_slice)
    }
}
