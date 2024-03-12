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

    pub(super) fn quadrant(&self, point: Vec2) -> Quadrant {
        let cmp = point.cmplt(self.mid);

        match (cmp.x, cmp.y) {
            (true, true) => Quadrant::TopLeft,
            (false, true) => Quadrant::TopRight,
            (true, false) => Quadrant::BottomLeft,
            (false, false) => Quadrant::BottomRight,
        }
    }

    pub(super) fn child(&self, quadrant: Quadrant) -> Self {
        match quadrant {
            Quadrant::TopLeft => Self::new(self.min, self.mid),
            Quadrant::TopRight => Self::new(
                Vec2::new(self.mid.x, self.min.y),
                Vec2::new(self.max.x, self.mid.y),
            ),
            Quadrant::BottomLeft => Self::new(
                Vec2::new(self.min.x, self.mid.y),
                Vec2::new(self.mid.x, self.max.y),
            ),
            Quadrant::BottomRight => Self::new(self.mid, self.max),
        }
    }
}

#[derive(Default)]
pub(super) struct Quadrants<T>([Option<T>; 4]);

impl<T> Quadrants<T> {
    pub(super) fn new(
        top_left: Option<T>,
        top_right: Option<T>,
        bottom_left: Option<T>,
        bottom_right: Option<T>,
    ) -> Self {
        Self([top_left, top_right, bottom_left, bottom_right])
    }

    pub(super) fn get(&self, quadrant: Quadrant) -> Option<&T> {
        self.0[self.index(quadrant)].as_ref()
    }

    pub(super) fn get_mut(&mut self, quadrant: Quadrant) -> Option<&mut T> {
        self.0[self.index(quadrant)].as_mut()
    }

    pub(super) fn set(&mut self, quadrant: Quadrant, mut value: Option<T>) -> Option<T> {
        std::mem::swap(&mut self.0[self.index(quadrant)], &mut value);
        value
    }

    pub(super) fn replace(&mut self, old: &T, new: Option<T>)
    where
        T: PartialEq,
    {
        for value in self.0.iter_mut() {
            if value.as_ref().map_or(false, |value| value.eq(old)) {
                *value = new;
                return;
            }
        }
    }

    pub(super) fn map<F, U>(self, mut f: F) -> Quadrants<U>
    where
        F: FnMut(T) -> U,
    {
        Quadrants(self.0.map(|l| match l {
            Some(l) => Some(f(l)),
            None => None,
        }))
    }

    fn index(&self, quadrant: Quadrant) -> usize {
        match quadrant {
            Quadrant::TopLeft => 0,
            Quadrant::TopRight => 1,
            Quadrant::BottomLeft => 2,
            Quadrant::BottomRight => 3,
        }
    }
}

impl<'a, T> IntoIterator for &'a Quadrants<T> {
    type Item = &'a T;
    type IntoIter = QuadrantsIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        QuadrantsIter {
            index: 0,
            items: &self.0,
        }
    }
}

pub(super) struct QuadrantsIter<'a, T> {
    index: usize,
    items: &'a [Option<T>; 4],
}

impl<'a, T> Iterator for QuadrantsIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.items.len() {
            let item = &self.items[self.index];
            self.index += 1;

            if item.is_some() {
                return item.as_ref();
            }
        }

        None
    }
}

#[derive(Clone, Copy)]
pub(super) enum Quadrant {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
