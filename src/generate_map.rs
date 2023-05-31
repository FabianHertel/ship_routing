use std::{time::SystemTime, fs, error::Error, f64::consts::PI};
use geojson::{GeoJson, Geometry, Value};
use rayon::prelude::*;

pub fn generate_map() -> Result<(), Box<dyn Error>> {

    println!("1/?: Read GeoJSONs parallel ...");
    let now = SystemTime::now();
    let coastlines: Vec<Vec<Vec<f64>>> = read_geojsons("complete");
    println!("1/?: Finished in {} sek", now.elapsed()?.as_secs());

    println!("2/?: Precalculations for islands/continents ...");
    let now = SystemTime::now();
    let islands: Vec<Island> = coastlines.iter().map(|e| Island::new(e.to_owned())).collect();
    println!("2/?: Biggest Continent: center={},{}; distance={}", islands[10].cog[0], islands[10].cog[1], islands[10].max_dist_from_cog);
    println!("2/?: Finished precalculations in {} sek", now.elapsed()?.as_secs());

    let now = SystemTime::now();
    println!("Point on land (Atlantic): {}", point_in_polygon_test(0.0,0.0, &islands));     // Atlantic
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!("Point on land (US): {}", point_in_polygon_test(-104.2758092369033, 34.117786526143604, &islands));      //US
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!("Point on land (North pole): {}", point_in_polygon_test(-27.24044854389621, 70.01752410356319, &islands));       // North of Grönland
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!("Point on land (Antarctica): {}", point_in_polygon_test(71.55, -74.1878186, &islands));      //Antarctica
    println!("Finished test in {} millis", now.elapsed()?.as_millis());

    Ok(())
}

struct Island {
    #[allow(dead_code)]
    coastline: Vec<Vec<f64>>,
    cog: Vec<f64>,
    max_dist_from_cog: f64
}

impl Island {
    fn new(coastline: Vec<Vec<f64>>) -> Island {
        let mut cog = [0.0, 0.0];
        for point in &coastline {
            cog = [cog[0] + point[0], cog[1] + point[1]];
        }
        let n = coastline.len().to_owned() as f64;
        cog = [cog[0] / n, cog[1] / n];

        let mut max_dist_from_cog = 0.0;
        for point in &coastline {
            let distance = distance_between(point, &cog.to_vec());
            if distance > max_dist_from_cog {
                max_dist_from_cog = distance;
            }
        }

        Island {
            coastline,
            cog: cog.to_vec(),
            max_dist_from_cog
        }
    }

    fn in_range(&self, x: &Vec<f64>) -> bool {
        return distance_between(x, &self.cog) < self.max_dist_from_cog
    }
}

fn distance_between(x: &Vec<f64>, y:&Vec<f64>) -> f64 {
    // from: http://www.movable-type.co.uk/scripts/latlong.html
    let φ1 = x[1] * PI/180.0; // φ, λ in radians
    let φ2 = y[1] * PI/180.0;
    let dφ = (y[1]-x[1]) * PI/180.0;
    let dλ = (y[0]-y[1]) * PI/180.0;
    const EARTH_RADIUS: f64 = 6371.0;

    let haversine = (dφ/2.0).sin().powi(2) + φ1.cos() * φ2.cos() * (dλ/2.0).sin().powi(2);
    let distance = EARTH_RADIUS * 2.0 * haversine.sqrt().atan2((1.0 - haversine).sqrt());
    return distance;
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
fn point_in_polygon_test(lon: f64, lat: f64, polygons: &Vec<Island>) -> bool {
    let mut n_of_checks = 0;

    for island in polygons {
        if island.in_range(&vec![lon, lat]) {
            let polygon = &island.coastline;
            let mut in_water = false;
            n_of_checks += 1;
            for j in 1..polygon.len() {       // ignore first point in polygon, because first and last will be the same
                let i = j - 1;
                if (polygon[i][0] > lon) != (polygon[j][0] > lon) {   // check if given point has lon between start and end point of edge
                    if (polygon[i][1] < lat) && (polygon[j][1] < lat) {     // if both start and end point are south, the going south will cross
                        println!("Line crossed: {}, {}; {}, {}", polygon[i][0], polygon[i][1], polygon[j][0], polygon[j][1]);
                        in_water = !in_water;
                    } else if (polygon[i][1] < lat) || (polygon[j][1] < lat) {      // if one of start and end point are south, we have to check... (happens rarely for coastline)
                        let slope = (lat-polygon[i][1])*(polygon[j][0]-polygon[i][0])-(polygon[j][1]-polygon[i][1])*(lon-polygon[i][0]);
                        if (slope < 0.0) != (polygon[j][0] < polygon[i][1]) {
                            println!("Line crossed (rare case!)");
                            in_water = !in_water;
                        }
                    }
                }
            }
            if in_water {
                println!("Found one after {} checks", n_of_checks);
                return true
            }
        }
    }
    println!("Didn't find after {} checks", n_of_checks);
    return false;
}