use glam::Vec2;

pub(super) struct Subdivisions {
    rectangle: Rectangle,
    target: Vec2,
}

impl Subdivisions {
    pub(super) fn new(rectangle: Rectangle, target: Vec2) -> Self {
        Self { rectangle, target }
    }
}

impl Iterator for Subdivisions {
    type Item = (Rectangle, Sector);

    fn next(&mut self) -> Option<Self::Item> {
        let sector = self.rectangle.sector(self.target);
        self.rectangle = sector.rectangle(self.rectangle);
        Some((self.rectangle, sector))
    }
}

pub(super) struct Rectangle {
    min: Vec2,
    max: Vec2,
}

impl Rectangle {
    pub(super) fn from_half_size(half_size: Vec2) -> Self {
        Self {
            min: -half_size,
            max: half_size,
        }
    }

    fn sector(&self, point: Vec2) -> Sector {
        let mut sector = Sector {
            midpoint: self.midpoint(),
            index: 0,
        };

        if sector.midpoint.x <= point.x {
            sector.index += 1;
        }
        if sector.midpoint.y <= point.y {
            sector.index += 2;
        }

        sector
    }

    fn midpoint(&self) -> Vec2 {
        self.max.lerp(self.min, 0.5)
    }
}

pub(super) struct Sector {
    midpoint: Vec2,
    index: usize,
}

impl Sector {
    pub(super) fn index(&self) -> usize {
        self.index
    }

    // TODO docs
    fn rectangle(&self, parent: Rectangle) -> Rectangle {
        let mut rectangle = Rectangle {
            min: parent.min,
            max: parent.max,
        };

        if self.index % 2 == 1 {
            rectangle.min.x = self.midpoint.x;
        } else {
            rectangle.max.x = self.midpoint.x;
        }

        if self.index > 1 {
            rectangle.min.y = self.midpoint.y;
        } else {
            rectangle.max.y = self.midpoint.y;
        }

        rectangle
    }
}
