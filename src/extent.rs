use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Extent2d {
    pub width: u32,
    pub height: u32,
}

impl Extent2d {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub const fn is_zero(&self) -> bool {
        self.width == 0 && self.height == 0
    }
}

impl fmt::Display for Extent2d {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Extent3d {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Extent3d {
    pub const ZERO: Self = Self::new(0, 0, 0);

    pub const fn new(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }

    pub const fn is_zero(&self) -> bool {
        self.width == 0 && self.height == 0 && self.depth == 0
    }

    pub const fn to_2d(self) -> Extent2d {
        Extent2d {
            width: self.width,
            height: self.height,
        }
    }
}

impl fmt::Display for Extent3d {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}x{}", self.width, self.height, self.depth)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Offset2d {
    pub x: i32,
    pub y: i32,
}

impl Offset2d {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const fn is_zero(&self) -> bool {
        self.x == 0 && self.y == 0
    }
}

impl fmt::Display for Offset2d {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Rect2d {
    pub offset: Offset2d,
    pub extent: Extent2d,
}

impl Rect2d {
    pub const ZERO: Self = Self::new(Offset2d::ZERO, Extent2d::ZERO);

    pub const fn new(offset: Offset2d, extent: Extent2d) -> Self {
        Self { offset, extent }
    }

    pub const fn from_size(width: u32, height: u32) -> Self {
        Self {
            offset: Offset2d::ZERO,
            extent: Extent2d::new(width, height),
        }
    }
}

impl From<Extent2d> for Rect2d {
    fn from(extent: Extent2d) -> Self {
        Self {
            offset: Offset2d::ZERO,
            extent,
        }
    }
}

impl fmt::Display for Rect2d {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} {}]", self.offset, self.extent)
    }
}
