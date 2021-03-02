macro_rules! gen_z_2 {
    {$($a: ident,)*;$($b: ident,)*} => {
        $(
            pub const $a: f32 = $b + 0.0001;
        )+
    }
}

macro_rules! gen_z {
    {$a: ident $($v:ident)*} => {
        pub const $a: f32 = 0.3;
        gen_z_2!{$($v,)*MAX_Z,;$a,$($v,)*}
    }
}

gen_z! {
    Z_GRID // 0.01
    Z_LOT // 0.3
    Z_INTER_BG
    Z_LANE_BG
    Z_LANE
    Z_SIDEWALK
    Z_ARROW
    Z_CROSSWALK
    Z_TREE_SHADOW
    Z_HOUSE
    Z_SIGNAL
    Z_CAR // 0.4
    Z_TREE
    Z_TOOL // 0.9
    Z_DEBUG_BG
    Z_DEBUG // 1.0
}
