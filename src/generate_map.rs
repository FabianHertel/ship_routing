use std::{time::SystemTime, fs, error::Error};
use geojson::{GeoJson, Geometry, Value};
use rayon::prelude::*;

pub fn generate_map() -> Result<(), Box<dyn Error>> {

    println!("1/?: Read GeoJSONs parallel ...");
    let now = SystemTime::now();
    let coastlines: Vec<Vec<Vec<f64>>> = read_geojsons("complete");
    println!("1/?: Finished in {} sek", now.elapsed()?.as_secs());

    let now = SystemTime::now();
    println!("Point in water (Atlantic): {}", point_in_polygon_test(0.0,0.0, &coastlines));     // Atlantic
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!("Point in water (US): {}", point_in_polygon_test(-104.2758092369033, 34.117786526143604, &coastlines));      //US
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!("Point in water (North pole): {}", point_in_polygon_test(-27.24044854389621, 70.01752410356319, &coastlines));       // North of Grönland
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!("Point in water (Antarctica): {}", point_in_polygon_test(71.55, -74.1878186, &coastlines));      //Antarctica
    println!("Finished test in {} millis", now.elapsed()?.as_millis());

    Ok(())
}

pub fn read_geojsons(prefix: &str) -> Vec<Vec<Vec<f64>>> {
    return  ["continents", "big_islands", "islands", "small_islands"].par_iter().map(|filename| {
        let now = SystemTime::now();
        let filename = prefix.to_owned() + "_" + filename;
        let geojson_str = fs::read_to_string(format!("./geojson/{filename}.json")).expect("Unable to read JSON file");
        let geojson: GeoJson = geojson_str.parse::<GeoJson>().unwrap();     // needs much of time (4-5min for world)
        println!("Parsing {} finished after {} sek", filename, now.elapsed().unwrap().as_secs());
        let geometry: Geometry = Geometry::try_from(geojson).unwrap();
        match geometry.value {
            Value::MultiLineString(coords) => coords,
            _ => vec![]
        }
    }).reduce(|| vec![], |mut a,mut b| {
        a.append(&mut b);
        return a
    });
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
                } else if (polygon[i][1] < lat) || (polygon[j][1] < lat) {      // if one of start and end point are south, we have to check... (happens rarely for coastline)
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