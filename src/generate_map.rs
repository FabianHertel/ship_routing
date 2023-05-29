use std::{time::SystemTime, fs, error::Error};

use geojson::{GeoJson, Geometry, Value};


pub fn generate_map() -> Result<(), Box<dyn Error>> {
    println!("1/?: Read GeoJSON ...");
    let now = SystemTime::now();
    let geojson_str = fs::read_to_string("./geojson.json").expect("Unable to read JSON file");
    let geojson: GeoJson = geojson_str.parse::<GeoJson>().unwrap();
    let geometry: Geometry = Geometry::try_from(geojson).unwrap();
    if let Value::MultiLineString(coords) = geometry.value {
        println!("Number of coords {}", coords.len())
    }
    println!("1/?: Finished in {} sek", now.elapsed()?.as_secs());

    // println!("Point in polygon test: {}", point_in_polygon_test(0.0,0.0, coastlines));

    Ok(())
}

// pub fn point_in_polygon_test(lon: f64, lat: f64, polygons: Vec<Vec<Vec<f64>>>) -> bool {
//     return true;
// }