
use std::{f64::consts::PI, fmt::{Display, Formatter}};

use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Coordinates(pub f64, pub f64);

impl Coordinates {
    pub fn from_vec(vector: &Vec<f64>) -> Coordinates {
        return Coordinates(vector[0], vector[1])
    }
    pub fn from_str(str: &str) -> Coordinates {
        let split:Vec<&str> = str.split(",").collect();
        return Coordinates(split[0].parse::<f64>().unwrap(), split[1].parse::<f64>().unwrap());
    }

    pub fn distance_to(&self, y: &Coordinates) -> f64 {
        return distance_between(self.0, self.1, y.0, y.1);
    }
}

impl Display for Coordinates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.0, self.1)
    }
}

pub fn distance_between(lon1:f64, lat1:f64, lon2:f64, lat2:f64) -> f64 {
    // from: http://www.movable-type.co.uk/scripts/latlong.html
    let φ1 = lat1 * PI/180.0; // φ, λ in radians
    let φ2 = lat2 * PI/180.0;
    let dφ = (lat2-lat1) * PI/180.0;
    let dλ = (lon2-lon1) * PI/180.0;
    const EARTH_RADIUS: f64 = 6371.0;

    let haversine = (dφ/2.0).sin().powi(2) + φ1.cos() * φ2.cos() * (dλ/2.0).sin().powi(2);
    let distance = EARTH_RADIUS * 2.0 * haversine.sqrt().atan2((1.0 - haversine).sqrt());
    return distance;
}