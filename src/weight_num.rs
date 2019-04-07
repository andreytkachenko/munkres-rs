use std::ops::{Add, Sub};
use std::{f32, f64};

pub trait WeightNum: PartialOrd + Copy + Sub<Output = Self> + Add<Output = Self> {
    fn is_zero(&self) -> bool;
    fn is_disallowed(&self) -> bool {
        false
    }
}

impl WeightNum for usize {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl WeightNum for isize {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl WeightNum for u64 {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl WeightNum for i64 {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl WeightNum for u32 {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl WeightNum for i32 {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl WeightNum for u16 {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl WeightNum for i16 {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl WeightNum for u8 {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl WeightNum for i8 {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl WeightNum for f64 {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0.0
    }

    fn is_disallowed(&self) -> bool {
        *self == f64::INFINITY
    }
}

impl WeightNum for f32 {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0.0
    }

    fn is_disallowed(&self) -> bool {
        *self == f32::INFINITY
    }
}