use crate::loader::FlightInfo;

pub struct Flight {
    min_height: f32,
    max_height: f32,
}

impl Flight {
    /// Returns minimum flight height (above terrain) of the object.
    pub fn min_height(&self) -> f32 {
        self.min_height
    }

    /// Returns maximum flight height (above terrain) of the object.
    pub fn max_height(&self) -> f32 {
        self.max_height
    }
}

impl From<&FlightInfo> for Flight {
    fn from(info: &FlightInfo) -> Self {
        Self {
            min_height: info.min_height(),
            max_height: info.max_height(),
        }
    }
}
