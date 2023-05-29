use std::{time::SystemTime, fs, error::Error};

use geojson::{GeoJson, Geometry, Value};


pub fn generate_map() -> Result<(), Box<dyn Error>> {
    println!("1/?: Read GeoJSON ...");
    let now = SystemTime::now();
    let geojson_str = fs::read_to_string("./geojson/import.json").expect("Unable to read JSON file");
    let geojson: GeoJson = geojson_str.parse::<GeoJson>().unwrap();
    let geometry: Geometry = Geometry::try_from(geojson).unwrap();
    let coastlines: Vec<Vec<Vec<f64>>> = match geometry.value {
        Value::MultiLineString(coords) => coords,
        _ => vec![]
    };
    println!("1/?: Finished in {} sek", now.elapsed()?.as_secs());

    println!("Point in polygon test: {}", point_in_polygon_test(0.0,0.0, &coastlines));
    println!("Point in polygon test: {}", point_in_polygon_test(34.117786526143604, -104.2758092369033, &coastlines));
    println!("Point in polygon test: {}", point_in_polygon_test(-27.24044854389621, 70.01752410356319, &coastlines));
    println!("Point in polygon test: {}", point_in_polygon_test(71.55, -74.1878186, &coastlines));

    Ok(())
}

/**
 * This method will check, the given point is inside water.
 * If so, true will be returned.
 * This will be done by checking how many coastlines will be crossed going to the southpole.
 * We consider the earth as a 2D map with (x,y) = (lon, lat).
 * So the southpole has the same width as the equator, which is fine for this algorithm.
 * From the given point we check how many coastlines are crossed going straight south.
 * If it is even, we are in the sea. If odd, we are on land.
 * Note: South pole seems to marked as water in OSM. Antartica seems to end there.
 */
fn point_in_polygon_test(lon: f64, lat: f64, polygons: &Vec<Vec<Vec<f64>>>) -> bool {
    let mut in_water = true;

    for polygon in polygons {
        for j in 1..polygon.len() {       // ignore first point in polygon, because first and last will be the same
            let i = j - 1;
            if (polygon[i][0] > lon) != (polygon[j][0] > lon) {   // check if given point has lon between start and end point of edge
                if (polygon[i][1] < lat) && (polygon[j][1] < lat) {     // if both start and end point are south, the going south will cross
                    println!("Line crossed: {}, {}; {}, {}", polygon[i][0], polygon[i][1], polygon[j][0], polygon[j][1]);
                    in_water = !in_water;
                } else if (polygon[i][1] < lat) || (polygon[j][1] < lat) {      // if one of start and end point are south, we have to check...
                    let slope = (lat-polygon[i][1])*(polygon[j][0]-polygon[i][0])-(polygon[j][1]-polygon[i][1])*(lon-polygon[i][0]);
                    if (slope < 0.0) != (polygon[j][0] < polygon[i][1]) {
                        println!("Line crossed");
                        in_water = !in_water;
                    }
                }
            }
        }
    }
    return in_water;
}