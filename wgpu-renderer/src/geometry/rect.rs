use scale::geometry::Vec2;

#[derive(Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[allow(dead_code)]
impl Rect {
    /// Create a new `Rect`.
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    /// Creates a new `Rect` a la Love2D's `love.graphics.newQuad`,
    /// as a fraction of the reference rect's size.
    pub fn fraction(x: f32, y: f32, w: f32, h: f32, reference: &Rect) -> Rect {
        Rect {
            x: x / reference.w,
            y: y / reference.h,
            w: w / reference.w,
            h: h / reference.h,
        }
    }

    /// Create a new rect from `i32` coordinates.
    pub const fn new_i32(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x: x as f32,
            y: y as f32,
            w: w as f32,
            h: h as f32,
        }
    }

    /// Create a new `Rect` with all values zero.
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Creates a new `Rect` at `0,0` with width and height 1.0
    pub const fn one() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }

    /// Returns the left edge of the `Rect`
    pub const fn left(&self) -> f32 {
        self.x
    }

    /// Returns the right edge of the `Rect`
    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    /// Returns the top edge of the `Rect`
    pub const fn top(&self) -> f32 {
        self.y
    }

    /// Returns the bottom edge of the `Rect`
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    /// Checks whether the `Rect` contains a `Point`
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y <= self.bottom()
            && point.y >= self.top()
    }

    /// Checks whether the `Rect` contains a `Point`
    pub fn contains_within(&self, point: Vec2, tolerance: f32) -> bool {
        point.x >= self.left() - tolerance
            && point.x <= self.right() + tolerance
            && point.y <= self.bottom() + tolerance
            && point.y >= self.top() - tolerance
    }

    /// Checks whether the `Rect` overlaps another `Rect`
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.top() <= other.bottom()
            && self.bottom() >= other.top()
    }

    pub fn intersects_line_within(&self, p1: Vec2, p2: Vec2, tolerance: f32) -> bool {
        let outcode0 = self.compute_code(p1, tolerance);
        let outcode1 = self.compute_code(p2, tolerance);
        if outcode0 == 0 || outcode1 == 0 {
            return true;
        }
        if outcode0 & outcode1 != 0 {
            return false;
        }
        true
    }

    fn compute_code(&self, p: Vec2, tolerance: f32) -> u8 {
        const INSIDE: u8 = 0; // 0000
        const LEFT: u8 = 1; // 0001
        const RIGHT: u8 = 2; // 0010
        const BOTTOM: u8 = 4; // 0100
        const TOP: u8 = 8; // 1000
        let mut code = INSIDE; // initialised as being inside of [[clip window]]
        let x = p.x;
        let y = p.y;

        if x < self.x - tolerance {
            // to the left of clip window
            code |= LEFT;
        } else if x > self.x + self.w + tolerance {
            // to the right of clip window
            code |= RIGHT;
        }

        if y < self.y - tolerance {
            // below the clip window
            code |= BOTTOM;
        } else if y > self.y + self.h + tolerance {
            // above the clip window
            code |= TOP;
        }
        code
    }
    /// Translates the `Rect` by an offset of (x, y)
    pub fn translate(&mut self, offset: Vec2) {
        self.x += offset.x;
        self.y += offset.y;
    }

    /// Moves the `Rect`'s origin to (x, y)
    pub fn move_to(&mut self, destination: Vec2) {
        self.x = destination.x;
        self.y = destination.y;
    }

    /// Scales the `Rect` by a factor of (sx, sy),
    /// growing towards the bottom-left
    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.w *= sx;
        self.h *= sy;
    }
    /// Returns a new `Rect` that includes all points of these two `Rect`s.
    pub fn combine_with(self, other: Rect) -> Rect {
        let x = f32::min(self.x, other.x);
        let y = f32::min(self.y, other.y);
        let w = f32::max(self.right(), other.right()) - x;
        let h = f32::max(self.bottom(), other.bottom()) - y;
        Rect { x, y, w, h }
    }
}
