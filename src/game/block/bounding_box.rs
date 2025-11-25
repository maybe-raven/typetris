use vector2d::Vector2D;

pub trait MixAdd<O> {
    fn mix_add(self, other: O) -> Self;
    fn mix_add_assign(&mut self, other: O)
    where
        Self: Sized + Copy,
    {
        *self = self.mix_add(other);
    }
}
impl MixAdd<i8> for u8 {
    #[inline]
    fn mix_add(self, other: i8) -> Self {
        if other < 0 {
            self.saturating_sub(other as u8)
        } else {
            self + other as u8
        }
    }
}
impl MixAdd<Vector2D<i8>> for Vector2D<u8> {
    #[inline]
    fn mix_add(self, other: Vector2D<i8>) -> Self {
        Self {
            x: self.x.mix_add(other.x),
            y: self.y.mix_add(other.y),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,
}

impl BoundingBox {
    #[inline]
    pub const fn new(x: u8, y: u8, width: u8, height: u8) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    #[inline]
    pub fn shift(&mut self, offset: Vector2D<i8>) {
        self.x.mix_add_assign(offset.x);
        self.y.mix_add_assign(offset.y);
    }
    #[inline]
    pub const fn intersect(self, other: Self) -> bool {
        !(self.x + self.width <= other.x
            || other.x + other.width <= self.x
            || self.y + self.height <= other.y
            || other.y + other.height <= self.y)
    }
    #[inline]
    pub const fn contains(self, point: Vector2D<u8>) -> bool {
        self.x <= point.x
            && self.x + self.width >= point.x
            && self.y <= point.y
            && self.y + self.height >= point.y
    }
}

#[cfg(test)]
mod tests {
    use super::BoundingBox;

    // Helper function to create a BoundingBox easily
    const fn bb(x: u8, y: u8, w: u8, h: u8) -> BoundingBox {
        BoundingBox::new(x, y, w, h)
    }

    // --- Basic Overlap Tests ---

    #[test]
    fn overlap_simple() {
        // b1: (0, 0, 10, 10)
        // b2: (5, 5, 10, 10) - Overlaps significantly
        let b1 = bb(0, 0, 10, 10);
        let b2 = bb(5, 5, 10, 10);
        assert!(b1.intersect(b2), "Simple overlap");
    }

    #[test]
    fn overlap_x_axis_only() {
        // b1: (0, 0, 10, 5)
        // b2: (5, 0, 10, 5) - Overlaps on X, same Y
        let b1 = bb(0, 0, 10, 5);
        let b2 = bb(5, 0, 10, 5);
        assert!(b1.intersect(b2), "Overlap on X-axis only");
    }

    #[test]
    fn overlap_y_axis_only() {
        // b1: (0, 0, 5, 10)
        // b2: (0, 5, 5, 10) - Overlaps on Y, same X
        let b1 = bb(0, 0, 5, 10);
        let b2 = bb(0, 5, 5, 10);
        assert!(b1.intersect(b2), "Overlap on Y-axis only");
    }

    // --- No Overlap Tests ---

    #[test]
    fn no_overlap_separated_x() {
        // b1: (0, 0, 10, 10)
        // b2: (11, 0, 10, 10) - Separated by 1 unit on X
        let b1 = bb(0, 0, 10, 10);
        let b2 = bb(11, 0, 10, 10);
        assert!(!b1.intersect(b2), "No overlap, separated on X");
    }

    #[test]
    fn no_overlap_separated_y() {
        // b1: (0, 0, 10, 10)
        // b2: (0, 11, 10, 10) - Separated by 1 unit on Y
        let b1 = bb(0, 0, 10, 10);
        let b2 = bb(0, 11, 10, 10);
        assert!(!b1.intersect(b2), "No overlap, separated on Y");
    }

    #[test]
    fn no_overlap_far_apart() {
        // b1: (0, 0, 5, 5)
        // b2: (100, 100, 5, 5) - Far apart
        let b1 = bb(0, 0, 5, 5);
        let b2 = bb(100, 100, 5, 5);
        assert!(!b1.intersect(b2), "No overlap, far apart");
    }

    // --- Edge Cases: Touching/Adjacency ---

    #[test]
    fn touching_x_edge() {
        // b1: (0, 0, 10, 10) -> Right edge is x=10
        // b2: (10, 0, 10, 10) -> Left edge is x=10. **The provided logic treats touching as non-overlapping.**
        let b1 = bb(0, 0, 10, 10);
        let b2 = bb(10, 0, 10, 10);
        // Center_x difference: |0 - 10| = 10
        // Combined half-width: (10 + 10) / 2 = 10
        // 10 * 2 < 20 -> 20 < 20, which is false.
        assert!(!b1.intersect(b2), "Touching edge on X (non-intersecting)");
    }

    #[test]
    fn touching_y_edge() {
        // b1: (0, 0, 10, 10) -> Bottom edge is y=10
        // b2: (0, 10, 10, 10) -> Top edge is y=10. **The provided logic treats touching as non-overlapping.**
        let b1 = bb(0, 0, 10, 10);
        let b2 = bb(0, 10, 10, 10);
        assert!(!b1.intersect(b2), "Touching edge on Y (non-intersecting)");
    }

    // --- Edge Cases: Containment/Identity ---

    #[test]
    fn exact_match() {
        // b1: (10, 10, 5, 5)
        // b2: (10, 10, 5, 5) - Identical
        let b1 = bb(10, 10, 5, 5);
        let b2 = bb(10, 10, 5, 5);
        assert!(b1.intersect(b2), "Exact match (self-intersection)");
    }

    #[test]
    fn internal_containment() {
        // b1: (0, 0, 20, 20)
        // b2: (5, 5, 10, 10) - B2 is fully inside B1
        let b1 = bb(0, 0, 20, 20);
        let b2 = bb(5, 5, 10, 10);
        assert!(b1.intersect(b2), "B2 contained within B1");
    }

    #[test]
    fn containment_at_origin() {
        // b1: (0, 0, 10, 10)
        // b2: (0, 0, 5, 5) - B2 is at B1's origin
        let b1 = bb(0, 0, 10, 10);
        let b2 = bb(0, 0, 5, 5);
        assert!(b1.intersect(b2), "B2 contained at B1's origin");
    }

    // --- Edge Cases: Minimal Overlap (1 unit) ---

    #[test]
    fn minimal_overlap_x() {
        // b1: (0, 0, 10, 10) -> Right edge is x=10
        // b2: (9, 0, 10, 10) -> Left edge is x=9. Overlap by 1 unit.
        let b1 = bb(0, 0, 10, 10);
        let b2 = bb(9, 0, 10, 10);
        // Center_x difference: |0 - 9| = 9
        // Combined half-width: (10 + 10) / 2 = 10
        // 9 * 2 < 20 -> 18 < 20, which is true.
        assert!(b1.intersect(b2), "Minimal 1-unit overlap on X");
    }

    #[test]
    fn minimal_overlap_y() {
        // b1: (0, 0, 10, 10)
        // b2: (0, 9, 10, 10) - Overlap by 1 unit on Y.
        let b1 = bb(0, 0, 10, 10);
        let b2 = bb(0, 9, 10, 10);
        assert!(b1.intersect(b2), "Minimal 1-unit overlap on Y");
    }

    // --- Edge Cases: Maximum u8 values (potential for wrap-around/overflow) ---
    // Note: Since you're using `abs_diff` and the comparison is `* 2 < (w1+w2)`,
    // overflow isn't an issue for this particular logic with u8s.

    #[test]
    fn max_u8_bounds_overlap() {
        // b1: (200, 200, 50, 50)
        // b2: (240, 240, 10, 10) - Overlap near the maximum value
        let b1 = bb(200, 200, 50, 50);
        let b2 = bb(240, 240, 10, 10);
        assert!(b1.intersect(b2), "Overlap near max u8 values");
    }

    #[test]
    fn max_u8_bounds_no_overlap() {
        // b1: (1, 1, 100, 100)
        // b2: (200, 200, 50, 50) - No overlap
        let b1 = bb(1, 1, 100, 100);
        let b2 = bb(200, 200, 50, 50);
        assert!(!b1.intersect(b2), "No overlap with large u8 values");
    }
}
