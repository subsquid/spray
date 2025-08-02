use std::debug_assert;


macro_rules! relation_mask {
    ($name:ident { $($rel:ident,)* }) => {
        #[derive(Default)]
        pub struct $name {
            mask: super::relation_mask::Bitmask
        }

        impl $name {
            super::relation_mask::_relation_mask_props!(0usize, $($rel)*);
        }

        impl super::item_filter::Relations for $name {
            fn include(&mut self, other: &Self) {
                self.mask.include(&other.mask)
            }
        }
    };
}
pub(super) use relation_mask;


macro_rules! _relation_mask_props {
    ($idx:expr) => {};
    ($idx:expr, $rel:ident) => {
        paste::paste! {
            pub fn [< set_ $rel >](&mut self, on: bool) {
                self.mask.set($idx, on)
            }

            pub fn [< has_ $rel >](&self) -> bool {
                self.mask.get($idx)
            }
        }
    };
    ($idx:expr, $rel:ident $($rest:ident)+) => {
        super::relation_mask::_relation_mask_props!($idx, $rel);
        super::relation_mask::_relation_mask_props!($idx + 1usize, $($rest)*);
    };
}
pub(super) use _relation_mask_props;


#[derive(Debug, Default, Clone)]
pub struct Bitmask {
    mask: u64
}


impl Bitmask {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, i: usize, on: bool) {
        if on {
            self.on(i)
        } else {
            self.off(i)
        }
    }

    pub fn on(&mut self, i: usize) {
        debug_assert!(i < 64);
        self.mask |= 1 << i;
    }

    pub fn off(&mut self, i: usize) {
        debug_assert!(i < 64);
        self.mask &= !(1 << i);
    }

    pub fn get(&self, i: usize) -> bool {
        debug_assert!(i < 64);
        self.mask & (1 << i) != 0
    }

    pub fn include(&mut self, other: &Bitmask) {
        self.mask |= other.mask;
    }
}