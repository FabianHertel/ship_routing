use std::{time::SystemTime, fs, error::Error, f64::consts::PI};
use geojson::{GeoJson, Geometry, Value};
use rayon::prelude::*;

pub fn generate_map() -> Result<(), Box<dyn Error>> {

    println!("1/?: Read GeoJSONs parallel ...");
    let now = SystemTime::now();
    let coastlines: Vec<Vec<Vec<f64>>> = read_geojsons("reduced");
    println!("1/?: Finished in {} sek", now.elapsed()?.as_secs());

    println!("2/?: Precalculations for islands/continents ...");
    let now = SystemTime::now();
    let islands: Vec<Island> = coastlines.iter().map(|e| Island::new(e.to_owned())).collect();
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
    let now = SystemTime::now();
    println!("Point on land (Mid of pacific): {}", point_in_polygon_test(-144.1294183409396, -47.75776979131451, &islands));      //Antarctica
    println!("Finished test in {} millis", now.elapsed()?.as_millis());

    Ok(())
}

/**
 * bounding box: [[min_lon, max_lon], [min_lat, max_lat]]
 */
struct Island {
    coastline: Vec<Vec<f64>>,
    bounding_box: [[f64; 2]; 2],
    reference_points: Vec<Vec<f64>>,
    max_dist_from_ref: f64
}

impl Island {
    fn new(coastline: Vec<Vec<f64>>) -> Island {
        let mut cog = [0.0, 0.0];
        let mut bounding_box = [[180.0, -180.0], [90.0, -90.0]];
        for point in &coastline {
            if point[0] < bounding_box[0][0] {
                bounding_box[0][0] = point[0];
            } if point[0] > bounding_box[0][1] {
                bounding_box[0][1] = point[0];
            } if point[1] < bounding_box[1][0] {
                bounding_box[1][0] = point[1];
            } if point[1] > bounding_box[1][1] {
                bounding_box[1][1] = point[1];
            }
            cog = [cog[0] + point[0], cog[1] + point[1]];
        }
        let n = coastline.len().to_owned() as f64;
        cog = [cog[0] / n, cog[1] / n];

        let mut reference_points = vec![cog.to_vec()];
        let mut most_far_away_point: &Vec<f64> = &vec![];
        let mut max_dist_from_ref = 0.0;
        let mut distance_sum = 0.0;
        for point in &coastline {
            let distance = distance_between(point, &cog.to_vec());
            distance_sum += distance;
            if distance > max_dist_from_ref {
                most_far_away_point = point;
                max_dist_from_ref = distance;
            }
        }
        
        // calculate more reference points (additional to cog) to get shorter max distances
        let mut refpoint_most_far_away = cog.to_vec();
        while max_dist_from_ref > 1000.0 && (distance_sum / coastline.len() as f64) < 0.5 * max_dist_from_ref {
            let new_refpoint = vec![(refpoint_most_far_away[0] + most_far_away_point[0]) / 2.0, (refpoint_most_far_away[1] + most_far_away_point[1]) / 2.0];
            print!("Dist: {}, with average: {} -> Generated {}, {}", max_dist_from_ref, (distance_sum / coastline.len() as f64), new_refpoint[0], new_refpoint[1]);
            reference_points.push(new_refpoint);
            distance_sum = 0.0;
            max_dist_from_ref = 0.0;
            for point in &coastline {
                let (distance, refpoint) = min_distance(point, &reference_points);
                distance_sum += distance;
                if distance > max_dist_from_ref {
                    most_far_away_point = point;
                    max_dist_from_ref = distance;
                    refpoint_most_far_away = refpoint;
                }
            }
            println!("; new dist: {}, new average: {}", max_dist_from_ref, (distance_sum / coastline.len() as f64));
        }

        Island {
                coastline,
                bounding_box,
                reference_points,
                max_dist_from_ref
            }
    }
}

fn min_distance(x: &Vec<f64>, reference_points:&Vec<Vec<f64>>) -> (f64, Vec<f64>) {
    let mut min_distance = 40000.0;
    let mut closest_refpoint = &reference_points[0];
    reference_points.iter().for_each(|reference_point| {
        let distance = distance_between(x, reference_point);
        if distance < min_distance {
            min_distance = distance;
            closest_refpoint = reference_point;
        }
    });
    return (min_distance, closest_refpoint.to_owned());
}

fn distance_between(x: &Vec<f64>, y:&Vec<f64>) -> f64 {
    // from: http://www.movable-type.co.uk/scripts/latlong.html
    let φ1 = x[1] * PI/180.0; // φ, λ in radians
    let φ2 = y[1] * PI/180.0;
    let dφ = (y[1]-x[1]) * PI/180.0;
    let dλ = (y[0]-x[0]) * PI/180.0;
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
    for island in polygons {
        let in_bounding_box = lon > island.bounding_box[0][0] && lon < island.bounding_box[0][1] && lat > island.bounding_box[1][0] && lat < island.bounding_box[1][1];
        if in_bounding_box {
            let in_range_of_ref_points = min_distance(&vec![lon, lat], &island.reference_points).0 < island.max_dist_from_ref;
            if in_range_of_ref_points {
                let polygon = &island.coastline;
                let mut in_water = false;
                // println!("Island center: {}, {}; max_dist_from_cog: {}; point distance: {}, coastline_points: {}", island.reference_points[0][0], island.reference_points[0][1], island.max_dist_from_cog, distance_between(&island.reference_points[0], &vec![lon, lat]), island.coastline.len());
                for j in 1..polygon.len() {       // ignore first point in polygon, because first and last will be the same
                    if (polygon[j-1][0] > lon) != (polygon[j][0] > lon) {   // check if given lon of point is between start and end point of edge
                        if (polygon[j-1][1] < lat) && (polygon[j][1] < lat) {     // if both start and end point are south, the going south will cross
                            // println!("Line crossed: {}, {}; {}, {}", polygon[i][0], polygon[i][1], polygon[j][0], polygon[j][1]);
                            in_water = !in_water;
                        } else if (polygon[j-1][1] < lat) || (polygon[j][1] < lat) {      // if one of start and end point are south, we have to check... (happens rarely for coastline)
                            let slope = (lat-polygon[j-1][1])*(polygon[j][0]-polygon[j-1][0])-(polygon[j][1]-polygon[j-1][1])*(lon-polygon[j-1][0]);
                            if (slope < 0.0) != (polygon[j][0] < polygon[j-1][1]) {
                                println!("Line crossed (rare case!)");
                                in_water = !in_water;
                            }
                        }
                    }
                }
                if in_water {
                    return true
                }
            } else {
                println!("Ref points saved checking {} edges of this continent: {}, {}!!!", island.coastline.len(), island.reference_points[0][0], island.reference_points[0][1]);
            }
        }
    }
    return false;
}