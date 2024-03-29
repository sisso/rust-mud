#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq, PartialOrd)]
pub struct V2I {
    pub x: i32,
    pub y: i32,
}

impl V2I {
    pub fn new(x: i32, y: i32) -> Self {
        V2I { x, y }
    }

    pub fn translate(&self, dx: i32, dy: i32) -> V2I {
        let new_x = self.x as i32 + dx;
        let new_y = self.y as i32 + dy;

        V2I::new(new_x, new_y)
    }

    pub fn as_array(&self) -> [i32; 2] {
        [self.x, self.y]
    }
}

impl From<(i32, i32)> for V2I {
    fn from((x, y): (i32, i32)) -> Self {
        V2I { x: x, y: y }
    }
}

impl From<[i32; 2]> for V2I {
    fn from(array: [i32; 2]) -> Self {
        V2I {
            x: array[0],
            y: array[1],
        }
    }
}

#[derive(Debug, Clone)]
pub struct RectI {
    topleft: V2I,
    bottomright: V2I,
}

impl RectI {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        RectI::new_2_points((x, y).into(), (x + w, y + h).into())
    }
    pub fn new_2_points(topleft: V2I, bottomright: V2I) -> Self {
        RectI {
            topleft,
            bottomright,
        }
    }

    pub fn get_top_left(&self) -> V2I {
        self.topleft.clone()
    }

    pub fn get_bottom_right(&self) -> V2I {
        self.bottomright.clone()
    }

    pub fn get_width(&self) -> i32 {
        self.bottomright.x - self.topleft.x
    }

    pub fn get_height(&self) -> i32 {
        self.bottomright.y - self.topleft.y
    }

    pub fn is_inside(&self, v: &V2I) -> bool {
        v.x >= self.topleft.x
            && v.x <= self.bottomright.x
            && v.y >= self.topleft.y
            && v.y <= self.bottomright.y
    }

    pub fn to_local(&self, v: &V2I) -> V2I {
        v.translate(-self.topleft.x, -self.topleft.y)
    }

    pub fn to_global(&self, v: &V2I) -> V2I {
        v.translate(self.topleft.x, self.topleft.y)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_recti_is_inside() {
        let r = RectI::new_2_points((-2, -1).into(), (2, 4).into());
        assert!(r.is_inside(&(-2, -1).into()));
        assert!(r.is_inside(&(0, 0).into()));
        assert!(r.is_inside(&(2, 4).into()));
        assert!(!r.is_inside(&(3, 4).into()));
        assert!(!r.is_inside(&(-2, -2).into()));
    }

    #[test]
    fn test_to_local() {
        let r = RectI::new(0, 0, 10, 10);
        assert_eq!(V2I::new(0, 0), r.to_local(&V2I::new(0, 0)));
        assert_eq!(V2I::new(1, 0), r.to_local(&V2I::new(1, 0)));
        assert_eq!(V2I::new(0, 1), r.to_local(&V2I::new(0, 1)));
        assert_eq!(V2I::new(-1, -1), r.to_local(&V2I::new(-1, -1)));

        let r = RectI::new(-10, -5, 10, 10);
        assert_eq!(V2I::new(10, 5), r.to_local(&V2I::new(0, 0)));
        assert_eq!(V2I::new(11, 5), r.to_local(&V2I::new(1, 0)));
        assert_eq!(V2I::new(10, 6), r.to_local(&V2I::new(0, 1)));
        assert_eq!(V2I::new(9, 4), r.to_local(&V2I::new(-1, -1)));

        let r = RectI::new(10, 5, 10, 10);
        assert_eq!(V2I::new(-10, -5), r.to_local(&V2I::new(0, 0)));
        assert_eq!(V2I::new(-9, -5), r.to_local(&V2I::new(1, 0)));
        assert_eq!(V2I::new(-10, -4), r.to_local(&V2I::new(0, 1)));
        assert_eq!(V2I::new(-11, -6), r.to_local(&V2I::new(-1, -1)));
    }
}
