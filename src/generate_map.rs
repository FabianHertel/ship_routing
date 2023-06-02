use std::{time::SystemTime, fs, error::Error, f64::consts::PI};
use geojson::{GeoJson, Geometry, Value};
use rayon::prelude::*;

use crate::datastructs::Coordinates;

pub fn generate_map() -> Result<(), Box<dyn Error>> {

    println!("1/?: Read GeoJSONs parallel ...");
    let now = SystemTime::now();
    let mut coastlines: Vec<Vec<Vec<f64>>> = read_geojsons("reduced");
    coastlines.sort_by(|a,b| b.len().cmp(&a.len()));
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
    println!("Point on land (Arktis): {}", point_in_polygon_test(-27.24044854389621, 70.01752410356319, &islands));       // North of Grönland
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!("Point on land (Antarctica): {}", point_in_polygon_test(71.55, -74.1878186, &islands));      //Antarctica
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!("Point on land (Mid of pacific): {}", point_in_polygon_test(-144.1294183409396, -47.75776979131451, &islands));
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!("Point on land (Russia): {}", point_in_polygon_test(82.35471714457248, 52.4548566256728, &islands));
    println!("Finished test in {} millis", now.elapsed()?.as_millis());


    Ok(())
}

/**
 * bounding box: [[min_lon, max_lon], [min_lat, max_lat]]
 */
pub struct Island {
    coastline: Vec<Coordinates>,
    bounding_box: [[f64;2]; 2],
    reference_points: Vec<Coordinates>,
    max_dist_from_ref: f64,
    lon_distribution: Vec<Vec<usize>>,
    lon_distribution_distance: f64,
}

impl std::fmt::Display for Island {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Islandcenter: ({})", self.reference_points[0])
    }
}

impl Island {

    fn new(coastline: Vec<Vec<f64>>) -> Island {
        let mut cog = Coordinates(0.0, 0.0);
        let mut bounding_box = [[180.0, -180.0], [90.0, -90.0]];
        let mut coastline_formatted: Vec<Coordinates> = vec![];
        let mut max_lon_jump = 0.0;
        for i in 0..coastline.len() {
            if coastline[i][0] < bounding_box[0][0] {
                bounding_box[0][0] = coastline[i][0];
            } if coastline[i][0] > bounding_box[0][1] {
                bounding_box[0][1] = coastline[i][0];
            } if coastline[i][1] < bounding_box[1][0] {
                bounding_box[1][0] = coastline[i][1];
            } if coastline[i][1] > bounding_box[1][1] {
                bounding_box[1][1] = coastline[i][1];
            }
            (cog.0, cog.1) = (cog.0 + coastline[i][0], cog.1 + coastline[i][1]);
            if i < coastline.len()-1 && (coastline[i+1][0] - coastline[i][0]).abs() > max_lon_jump {
                max_lon_jump = (coastline[i+1][0] - coastline[i][0]).abs();
            }
            coastline_formatted.push(Coordinates::from_vec(&coastline[i]));
        }
        const MIN_LON_DISTR_DIFF: f64 = 0.2;
        let lon_distribution_distance = max(max_lon_jump, MIN_LON_DISTR_DIFF);
        let n = coastline_formatted.len().to_owned() as f64;
        (cog.0, cog.1) = (cog.0 / n, cog.1 / n);
        
        let mut lon_distribution: Vec<Vec<usize>> = vec![];
        const MIN_SIZE_FOR_LON_DISTR: usize = 1000;
        if coastline.len() > MIN_SIZE_FOR_LON_DISTR && (bounding_box[0][1]-bounding_box[0][0]) > 10.0 * lon_distribution_distance {
            let n_seperations = ((bounding_box[0][1]-bounding_box[0][0]) / lon_distribution_distance).ceil() as usize;
            // println!("Max jump: {}; distr size: {}", max_lon_jump, n_seperations);
            lon_distribution = vec![vec![]; n_seperations];
        }

        let mut reference_points = vec![cog.clone()];
        let mut most_far_away_point = &Coordinates(0.0, 0.0);
        let mut max_dist_from_ref = 0.0;
        let mut distance_sum = 0.0;
        for i in 0..coastline_formatted.len() {
            let distance = distance_between(&coastline_formatted[i], &cog);
            distance_sum += distance;
            if distance > max_dist_from_ref {
                most_far_away_point = &coastline_formatted[i];
                max_dist_from_ref = distance;
            }
            if coastline.len() > MIN_SIZE_FOR_LON_DISTR && (bounding_box[0][1]-bounding_box[0][0]) > 10.0 * lon_distribution_distance {
                let index_in_lon_distr = ((coastline_formatted[i].0 - bounding_box[0][0]) / lon_distribution_distance).floor() as usize;
                lon_distribution[index_in_lon_distr].push(i);
            }
        }
        
        // calculate more reference points (additional to cog) to get shorter max distances
        let mut refpoint_most_far_away = cog;
        while max_dist_from_ref > 1000.0 && (distance_sum / coastline_formatted.len() as f64) < 0.5 * max_dist_from_ref {
            let new_refpoint = Coordinates((refpoint_most_far_away.0 + most_far_away_point.0) / 2.0, (refpoint_most_far_away.1 + most_far_away_point.1) / 2.0);
            print!("Dist: {}, with average: {} -> Generated {}, {}", max_dist_from_ref, (distance_sum / coastline_formatted.len() as f64), new_refpoint.0, new_refpoint.1);
            reference_points.push(new_refpoint);
            distance_sum = 0.0;
            max_dist_from_ref = 0.0;
            for point in &coastline_formatted {
                let (distance, refpoint) = min_distance(point, &reference_points);
                distance_sum += distance;
                if distance > max_dist_from_ref {
                    most_far_away_point = point;
                    max_dist_from_ref = distance;
                    refpoint_most_far_away = refpoint;
                }
            }
            println!("; new dist: {}, new average: {}", max_dist_from_ref, (distance_sum / coastline_formatted.len() as f64));
        }

        Island {
                coastline: coastline_formatted,
                bounding_box,
                reference_points,
                max_dist_from_ref,
                lon_distribution,
                lon_distribution_distance
            }
    }
}

fn max(v1: f64, v2: f64) -> f64 {
    if v1 > v2 {
        return v1;
    } else {
        return v2;
    }
}

fn min_distance(x: &Coordinates, reference_points:&Vec<Coordinates>) -> (f64, Coordinates) {
    let mut min_distance = 40000.0;
    let mut closest_refpoint = &reference_points[0];
    reference_points.iter().for_each(|reference_point| {
        let distance = distance_between(x, reference_point);
        if distance < min_distance {
            min_distance = distance;
            closest_refpoint = reference_point;
        }
    });
    return (min_distance, closest_refpoint.clone());
}

fn distance_between(x: &Coordinates, y:&Coordinates) -> f64 {
    // from: http://www.movable-type.co.uk/scripts/latlong.html
    let φ1 = x.1 * PI/180.0; // φ, λ in radians
    let φ2 = y.1 * PI/180.0;
    let dφ = (y.1-x.1) * PI/180.0;
    let dλ = (y.0-x.0) * PI/180.0;
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
 * This will be done by checking how many coastlines will be crossed going to the northpole.
 * We consider the earth as a 2D map with (x,y) = (lon, lat).
 * So the northpole has the same width as the equator, which is fine for this algorithm since coastlines are short.
 * According to our measurements of the coastlines with the table from https://dataverse.jpl.nasa.gov/dataset.xhtml?persistentId=hdl:2014/41271 there is a maximum error of single meters.
 * From the given point we check how many coastlines are crossed going straight north.
 * If it is even, we are in the sea. If odd, we are on land.
 * Note: Antartica avoids -180 to 180 edge, so coastline goes to the southpole and around it.
 */
fn point_in_polygon_test(lon: f64, lat: f64, polygons: &Vec<Island>) -> bool {
    for island in polygons {
        let in_bounding_box = lon > island.bounding_box[0][0] && lon < island.bounding_box[0][1] && lat > island.bounding_box[1][0] && lat < island.bounding_box[1][1];
        if in_bounding_box {
            let in_range_of_ref_points = min_distance(&Coordinates(lon, lat), &island.reference_points).0 < island.max_dist_from_ref;
            if in_range_of_ref_points {
                // println!("Island center: {}; max_dist_from_ref: {}; point distance: {}, coastline_points: {}", island, island.max_dist_from_ref, distance_between(&island.reference_points[0], &Coordinates(lon, lat)), island.coastline.len());
                let mut in_water = false;
                let polygon = &island.coastline;
                if island.lon_distribution.len() > 0 {
                    let index_in_lon_distr = ((lon - island.bounding_box[0][0]) / island.lon_distribution_distance).floor() as usize;
                    let mut last_point_i: usize = 0;
                    for point_i in &island.lon_distribution[index_in_lon_distr] {
                        if *point_i != last_point_i + 1 && *point_i > 0 {   // check edge before only if not checked before
                            let (start, end) = (&polygon[point_i-1], &polygon[*point_i]) ;
                            if (start.0 > lon) != (end.0 > lon) {   // check if given lon of point is between start and end point of edge
                                if (start.1 > lat) && (end.1 > lat) {     // if both start and end point are north, the going north will cross
                                    // println!("Line crossed: {}; {}", start, end);
                                    in_water = !in_water;
                                } else if (start.1 > lat) || (end.1 > lat) {      // if one of start and end point are south, we have to check... (happens rarely for coastline)
                                    let slope = (lat-start.1)*(end.0-start.0)-(end.1-start.1)*(lon-start.0);
                                    if (slope < 0.0) != (end.0 < start.0) {
                                        println!("Line crossed (rare case!)");
                                        in_water = !in_water;
                                    }
                                }
                            }
                        }
                        // check edge after always point is not the last one
                        if *point_i < island.coastline.len() - 1 {
                            let (start, end) = (&polygon[*point_i], &polygon[point_i+1]) ;
                            if (start.0 > lon) != (end.0 > lon) {   // check if given lon of point is between start and end point of edge
                                if (start.1 > lat) && (end.1 > lat) {     // if both start and end point are north, the going north will cross
                                    // println!("Line crossed: {}; {}", start, end);
                                    in_water = !in_water;
                                } else if (start.1 > lat) || (end.1 > lat) {      // if one of start and end point are south, we have to check... (happens rarely for coastline)
                                    let slope = (lat-start.1)*(end.0-start.0)-(end.1-start.1)*(lon-start.0);
                                    if (slope < 0.0) != (end.0 < start.0) {
                                        println!("Line crossed (rare case!)");
                                        in_water = !in_water;
                                    }
                                }
                            }
                        }
                        last_point_i = *point_i;
                    }
                } else {
                    for j in 1..polygon.len() {       // ignore first point in polygon, because first and last will be the same
                        let (start, end) = (&polygon[j-1], &polygon[j]);
                        if (start.0 > lon) != (end.0 > lon) {   // check if given lon of point is between start and end point of edge
                            if (start.1 > lat) && (end.1 > lat) {     // if both start and end point are north, the going north will cross
                                // println!("Line crossed: {}; {}", start, end);
                                in_water = !in_water;
                            } else if (start.1 > lat) || (end.1 > lat) {      // if one of start and end point are south, we have to check... (happens rarely for coastline)
                                let slope = (lat-start.1)*(end.0-start.0)-(end.1-start.1)*(lon-start.0);
                                if (slope < 0.0) != (end.0 < start.0) {
                                    println!("Line crossed (rare case!)");
                                    in_water = !in_water;
                                }
                            }
                        }
                    }
                }
                if in_water {
                    return true
                }
            } else {
                println!("Ref points saved checking {} edges of this continent: {}, {}!!!", island.coastline.len(), island.reference_points[0].0, island.reference_points[0].1);
            }
        }
    }
    return false;
}