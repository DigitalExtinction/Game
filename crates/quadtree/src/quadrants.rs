use glam::Vec2;

#[derive(Clone)]
pub(super) struct Rect {
    min: Vec2,
    max: Vec2,
    mid: Vec2,
}

impl Rect {
    pub(super) fn new(min: Vec2, max: Vec2) -> Self {
        let mid = 0.5 * (min + max);
        Self { min, max, mid }
    }

    // TODO use an enum instead
    pub(super) fn quadrant(&self, point: Vec2) -> usize {
        let cmp = point.cmplt(self.mid);
        (if cmp.x { 0 } else { 1 }) + (if cmp.y { 0 } else { 2 })
    }

    pub(super) fn child(&self, quadrant: usize) -> Self {
        match quadrant {
            0 => Self::new(self.min, self.mid),
            1 => Self::new(
                Vec2::new(self.mid.x, self.min.y),
                Vec2::new(self.max.x, self.mid.y),
            ),
            2 => Self::new(
                Vec2::new(self.min.x, self.mid.y),
                Vec2::new(self.mid.x, self.max.y),
            ),
            3 => Self::new(self.mid, self.max),
            _ => panic!("Invalid corner"),
        }
    }
}
