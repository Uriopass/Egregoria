use geom::Vec3;

/// Trait is needed to use bytemuck's conversion on external types
pub trait ToU8Slice {
    fn to_slice(&self) -> &[u8];
}

macro_rules! u8slice_impl {
    ($t: ty) => {
        unsafe impl bytemuck::Pod for $t {}
        unsafe impl bytemuck::Zeroable for $t {}
        impl crate::ToU8Slice for [$t] {
            fn to_slice(&self) -> &[u8] {
                bytemuck::cast_slice(self)
            }
        }
    };
}

macro_rules! u8slice_impl_external {
    ($nt: ident, $t: ty) => {
        #[repr(transparent)]
        #[derive(Clone, Copy)]
        struct $nt($t);

        unsafe impl bytemuck::Pod for $nt {}
        unsafe impl bytemuck::Zeroable for $nt {}

        impl ToU8Slice for [$t] {
            fn to_slice<'a>(&'a self) -> &'a [u8] {
                let v: &'a [$nt] = unsafe { &*(self as *const [$t] as *const [$nt]) };
                bytemuck::cast_slice(v)
            }
        }
    };
}

u8slice_impl_external!(Matrix4NT, mint::ColumnMatrix4<f32>);
u8slice_impl_external!(Vec3NT, Vec3);

impl ToU8Slice for [f32] {
    fn to_slice(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
}

impl ToU8Slice for [[f32; 4]] {
    fn to_slice(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
}
