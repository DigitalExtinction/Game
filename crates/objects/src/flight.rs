use serde::{Deserialize, Serialize};

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

impl TryFrom<FlightSerde> for Flight {
    type Error = anyhow::Error;

    fn try_from(flight_serde: FlightSerde) -> Result<Self, Self::Error> {
        Ok(Self {
            min_height: flight_serde.min_height,
            max_height: flight_serde.max_height,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct FlightSerde {
    min_height: f32,
    max_height: f32,
}
