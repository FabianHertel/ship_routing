use std::{time::SystemTime, fs::{self, File}, error::Error, io::{Write}};
use geojson::{GeoJson, Geometry, Value};
use rayon::{prelude::*};
use rand::Rng;

use crate::coordinates::{Coordinates};
use crate::graph::{Node, Edge};

pub fn generate_map() -> Result<(), Box<dyn Error>> {

    println!("1/?: Read GeoJSONs parallel ...");
    let now = SystemTime::now();
    let mut coastlines: Vec<Vec<Vec<f64>>> = read_geojsons("reduced");
    coastlines.sort_by(|a, b| b.len().cmp(&a.len()));
    println!("1/?: Finished in {} sek", now.elapsed()?.as_secs());

    println!("2/?: Precalculations for islands/continents ...");
    let now = SystemTime::now();
    let islands: Vec<Island> = coastlines
        .iter()
        .map(|e| Island::new(e.to_owned()))
        .collect();
    println!(
        "2/?: Finished precalculations in {} sek",
        now.elapsed()?.as_secs()
    );

    random_points_on_sphere(&islands);

    let now = SystemTime::now();
    println!(
        "Point on land (Atlantic): {}",
        point_in_polygon_test(0.0, 0.0, &islands)
    ); // Atlantic
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!(
        "Point on land (US): {}",
        point_in_polygon_test(-104.2758092369033, 34.117786526143604, &islands)
    ); //US
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!(
        "Point on land (Arktis): {}",
        point_in_polygon_test(-27.24044854389621, 70.01752410356319, &islands)
    ); // North of Grönland
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!(
        "Point on land (Antarctica): {}",
        point_in_polygon_test(71.55, -74.1878186, &islands)
    ); //Antarctica
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!(
        "Point on land (Mid of pacific): {}",
        point_in_polygon_test(-144.1294183409396, -47.75776979131451, &islands)
    );
    println!("Finished test in {} millis", now.elapsed()?.as_millis());
    let now = SystemTime::now();
    println!(
        "Point on land (Russia): {}",
        point_in_polygon_test(82.35471714457248, 52.4548566256728, &islands)
    );
    println!("Finished test in {} millis", now.elapsed()?.as_millis());

    Ok(())
}

/**
 * bounding box: [[min_lon, max_lon], [min_lat, max_lat]]
 */
pub struct Island {
    coastline: Vec<Coordinates>,
    bounding_box: [[f64; 2]; 2],
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
            }
            if coastline[i][0] > bounding_box[0][1] {
                bounding_box[0][1] = coastline[i][0];
            }
            if coastline[i][1] < bounding_box[1][0] {
                bounding_box[1][0] = coastline[i][1];
            }
            if coastline[i][1] > bounding_box[1][1] {
                bounding_box[1][1] = coastline[i][1];
            }
            (cog.0, cog.1) = (cog.0 + coastline[i][0], cog.1 + coastline[i][1]);
            if i < coastline.len() - 1
                && (coastline[i + 1][0] - coastline[i][0]).abs() > max_lon_jump
            {
                max_lon_jump = (coastline[i + 1][0] - coastline[i][0]).abs();
            }
            coastline_formatted.push(Coordinates::from_vec(&coastline[i]));
        }
        const MIN_LON_DISTR_DIFF: f64 = 0.2;
        let lon_distribution_distance = max(max_lon_jump, MIN_LON_DISTR_DIFF);
        let n = coastline_formatted.len().to_owned() as f64;
        (cog.0, cog.1) = (cog.0 / n, cog.1 / n);

        let mut lon_distribution: Vec<Vec<usize>> = vec![];
        const MIN_SIZE_FOR_LON_DISTR: usize = 1000;
        if coastline.len() > MIN_SIZE_FOR_LON_DISTR
            && (bounding_box[0][1] - bounding_box[0][0]) > 10.0 * lon_distribution_distance
        {
            let n_seperations = ((bounding_box[0][1] - bounding_box[0][0])
                / lon_distribution_distance)
                .ceil() as usize;
            // println!("Max jump: {}; distr size: {}", max_lon_jump, n_seperations);
            lon_distribution = vec![vec![]; n_seperations];
        }

        let mut reference_points = vec![cog.clone()];
        let mut most_far_away_point = &Coordinates(0.0, 0.0);
        let mut max_dist_from_ref = 0.0;
        let mut distance_sum = 0.0;
        for i in 0..coastline_formatted.len() {
            let distance = coastline_formatted[i].distance_to(&cog);
            distance_sum += distance;
            if distance > max_dist_from_ref {
                most_far_away_point = &coastline_formatted[i];
                max_dist_from_ref = distance;
            }
            if coastline.len() > MIN_SIZE_FOR_LON_DISTR
                && (bounding_box[0][1] - bounding_box[0][0]) > 10.0 * lon_distribution_distance
            {
                let index_in_lon_distr = ((coastline_formatted[i].0 - bounding_box[0][0])
                    / lon_distribution_distance)
                    .floor() as usize;
                lon_distribution[index_in_lon_distr].push(i);
            }
        }

        // calculate more reference points (additional to cog) to get shorter max distances
        let mut refpoint_most_far_away = cog;
        while max_dist_from_ref > 1000.0
            && (distance_sum / coastline_formatted.len() as f64) < 0.5 * max_dist_from_ref
        {
            let new_refpoint = Coordinates(
                (refpoint_most_far_away.0 + most_far_away_point.0) / 2.0,
                (refpoint_most_far_away.1 + most_far_away_point.1) / 2.0,
            );
            print!(
                "Dist: {}, with average: {} -> Generated {}, {}",
                max_dist_from_ref,
                (distance_sum / coastline_formatted.len() as f64),
                new_refpoint.0,
                new_refpoint.1
            );
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
            println!(
                "; new dist: {}, new average: {}",
                max_dist_from_ref,
                (distance_sum / coastline_formatted.len() as f64)
            );
        }

        Island {
            coastline: coastline_formatted,
            bounding_box,
            reference_points,
            max_dist_from_ref,
            lon_distribution,
            lon_distribution_distance,
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

fn min_distance(x: &Coordinates, reference_points: &Vec<Coordinates>) -> (f64, Coordinates) {
    let mut min_distance = 40000.0;
    let mut closest_refpoint = &reference_points[0];
    reference_points.iter().for_each(|reference_point| {
        let distance = x.distance_to(reference_point);
        if distance < min_distance {
            min_distance = distance;
            closest_refpoint = reference_point;
        }
    });
    return (min_distance, closest_refpoint.clone());
}

pub fn read_geojsons(prefix: &str) -> Vec<Vec<Vec<f64>>> {
    return ["continents", "big_islands", "islands", "small_islands"]
        .par_iter()
        .map(|filename| {
            let now = SystemTime::now();
            let filename = prefix.to_owned() + "_" + filename;
            let geojson_str = fs::read_to_string(format!("./geojson/{filename}.json"))
                .expect(&format!("Unable to read JSON file {}", filename));
            let geojson: GeoJson = geojson_str.parse::<GeoJson>().unwrap(); // needs much of time (4-5min for world)
            println!(
                "Parsing {} finished after {} sek",
                filename,
                now.elapsed().unwrap().as_secs()
            );
            let geometry: Geometry = Geometry::try_from(geojson).unwrap();
            match geometry.value {
                Value::MultiLineString(coords) => coords,
                _ => vec![],
            }
        })
        .reduce(
            || vec![],
            |mut a, mut b| {
                a.append(&mut b);
                return a;
            },
        );
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
        let in_bounding_box = lon > island.bounding_box[0][0]
            && lon < island.bounding_box[0][1]
            && lat > island.bounding_box[1][0]
            && lat < island.bounding_box[1][1];
        if in_bounding_box {
            let in_range_of_ref_points =
                min_distance(&Coordinates(lon, lat), &island.reference_points).0
                    < island.max_dist_from_ref;
            if in_range_of_ref_points {
                // println!("Island center: {}; max_dist_from_ref: {}; point distance: {}, coastline_points: {}", island, island.max_dist_from_ref, distance_between(&island.reference_points[0], &Coordinates(lon, lat)), island.coastline.len());
                let mut in_water = false;
                let polygon = &island.coastline;
                if island.lon_distribution.len() > 0 {
                    let index_in_lon_distr = ((lon - island.bounding_box[0][0])
                        / island.lon_distribution_distance)
                        .floor() as usize;
                    let mut last_point_i: usize = 0;
                    for point_i in &island.lon_distribution[index_in_lon_distr] {
                        if *point_i != last_point_i + 1 && *point_i > 0 {
                            // check edge before only if not checked before
                            let (start, end) = (&polygon[point_i - 1], &polygon[*point_i]);
                            if (start.0 > lon) != (end.0 > lon) {
                                // check if given lon of point is between start and end point of edge
                                if (start.1 > lat) && (end.1 > lat) {
                                    // if both start and end point are north, the going north will cross
                                    // println!("Line crossed: {}; {}", start, end);
                                    in_water = !in_water;
                                } else if (start.1 > lat) || (end.1 > lat) {
                                    // if one of start and end point are south, we have to check... (happens rarely for coastline)
                                    let slope = (lat - start.1) * (end.0 - start.0)
                                        - (end.1 - start.1) * (lon - start.0);
                                    if (slope < 0.0) != (end.0 < start.0) {
                                        // println!("Line crossed (rare case!)");
                                        in_water = !in_water;
                                    }
                                }
                            }
                        }
                        // check edge after always point is not the last one
                        if *point_i < island.coastline.len() - 1 {
                            let (start, end) = (&polygon[*point_i], &polygon[point_i + 1]);
                            if (start.0 > lon) != (end.0 > lon) {
                                // check if given lon of point is between start and end point of edge
                                if (start.1 > lat) && (end.1 > lat) {
                                    // if both start and end point are north, the going north will cross
                                    // println!("Line crossed: {}; {}", start, end);
                                    in_water = !in_water;
                                } else if (start.1 > lat) || (end.1 > lat) {
                                    // if one of start and end point are south, we have to check... (happens rarely for coastline)
                                    let slope = (lat - start.1) * (end.0 - start.0)
                                        - (end.1 - start.1) * (lon - start.0);
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
                    for j in 1..polygon.len() {
                        // ignore first point in polygon, because first and last will be the same
                        let (start, end) = (&polygon[j - 1], &polygon[j]);
                        if (start.0 > lon) != (end.0 > lon) {
                            // check if given lon of point is between start and end point of edge
                            if (start.1 > lat) && (end.1 > lat) {
                                // if both start and end point are north, the going north will cross
                                // println!("Line crossed: {}; {}", start, end);
                                in_water = !in_water;
                            } else if (start.1 > lat) || (end.1 > lat) {
                                // if one of start and end point are south, we have to check... (happens rarely for coastline)
                                let slope = (lat - start.1) * (end.0 - start.0)
                                    - (end.1 - start.1) * (lon - start.0);
                                if (slope < 0.0) != (end.0 < start.0) {
                                    // println!("Line crossed (rare case!)");
                                    in_water = !in_water;
                                }
                            }
                        }
                    }
                }
                if in_water {
                    return true;
                }
            } else {
                //println!("Ref points saved checking {} edges of this continent: {}, {}!!!", island.coastline.len(), island.reference_points[0].0, island.reference_points[0].1);
            }
        }
    }
    return false;
}

fn random_points_on_sphere(polygons: &Vec<Island>) -> () {
    let mut rng = rand::thread_rng();
    let mut new_node: Node;
    let mut points: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut x: f64;
    let mut y: f64;
    let mut z: f64;
    let mut lat: f64;
    let mut lon: f64;
    let mut id = 0;
    let mut counter = 0;
    let mut grid: Vec<Vec<Vec<Node>>> = vec![vec![Vec::new(); 180]; 360];
    let mut data = String::new();
    let max_distance = 30.0;

    while counter < 1000000 {
        x = 0.0;
        y = 0.0;
        z = 0.0;

        while ((x * x + y * y + z * z).sqrt()) < 0.001 {
            x = rng.gen_range(-1.0..1.0);
            y = rng.gen_range(-1.0..1.0);
            z = rng.gen_range(-1.0..1.0);
        }
        let norm = (x * x + y * y + z * z).sqrt();

        x = x / norm;
        y = y / norm;
        z = z / norm;

        lat = z.asin().to_degrees(); // asin(Z/R)
        lon = y.atan2(x).to_degrees(); // atan2(y,x)

        if norm <= 1.0 {
            if !point_in_polygon_test(lon, lat, polygons) {
                new_node = Node {
                    id: 0,
                    lat: lat,
                    lon: lon,
                };

                grid[(lon.round() + 180.0 - 1.0) as usize][(lat.round() + 90.0 - 1.0) as usize]
                    .push(new_node);
                counter = counter + 1;
                //print!("[{},{}],", lon, lat);
            }
        }
    }

    // set ids
    for i in 0..360 {
        for j in 0..180 {
            for k in 0..grid[i][j].len() {
                grid[i][j][k].id = id;
                points.push(grid[i][j][k].clone());
                id = id + 1;
            }
        }
    }

    for i in 0..360 {
        for j in 0..180 {
            if grid[i][j].len() >= 1 {
                let mut closest_ne = 0;
                let mut distance_ne = 40000.0;
                let mut closest_se = 0;
                let mut distance_se = 40000.0;
                let mut temp_nodes: Vec<Node> = Vec::new();
                let mut distance_to_node = 40000.0;

                temp_nodes.extend(&grid[i][j]);
                temp_nodes.extend(
                    &grid[(i + 1).rem_euclid(360)][((j as i32) - 1).rem_euclid(180) as usize],
                );
                temp_nodes.extend(&grid[(i + 1).rem_euclid(360)][(j).rem_euclid(180)]);
                temp_nodes.extend(&grid[(i + 1).rem_euclid(360)][(j + 1).rem_euclid(180)]);
                temp_nodes
                    .extend(&grid[(i).rem_euclid(360)][((j as i32) - 1).rem_euclid(180) as usize]);
                temp_nodes.extend(&grid[(i).rem_euclid(360)][(j + 1).rem_euclid(180)]);

                for k in &grid[i][j] {
                    for l in &temp_nodes {
                        if k.id != l.id {
                            distance_to_node = k.distance_to(&Coordinates(l.lon, l.lat));
                            if distance_to_node < max_distance {
                                //NORTH EAST
                                if k.lon < l.lon && k.lat < l.lat {
                                    if distance_to_node < distance_ne {
                                        distance_ne = distance_to_node;
                                        closest_ne = l.id;
                                    }
                                }
                                // SOUTH EAST
                                else if k.lon < l.lon && k.lat > l.lat {
                                    if distance_to_node < distance_se {
                                        distance_se = distance_to_node;
                                        closest_se = l.id;
                                    }
                                }
                            }
                        }
                    }
                    //create Edge for ne
                    if distance_ne < max_distance {
                        edges.push(Edge {
                            src: k.id,
                            tgt: closest_ne,
                            dist: distance_ne,
                        });
                        edges.push(Edge {
                            src: closest_ne,
                            tgt: k.id,
                            dist: distance_ne,
                        });
                    }

                    //Create Edge for se
                    if distance_se < max_distance {
                        edges.push(Edge {
                            src: k.id,
                            tgt: closest_se,
                            dist: distance_se,
                        });
                        edges.push(Edge {
                            src: closest_se,
                            tgt: k.id,
                            dist: distance_se,
                        });
                    }
                    distance_ne = 40000.0;
                    distance_se = 40000.0;
                }
            }
        }
    }

    edges.sort_by(|a, b| a.src.cmp(&b.src));

    data = data + &points.len().to_string() + "\n";
    data = data + &edges.len().to_string() + "\n";

    for node in &points {
        data = data
            + &node.id.to_string()
            + " "
            + &node.lat.to_string()
            + " "
            + &node.lon.to_string()
            + "\n";
    }
    for edge in &edges {
        data = data
            + &edge.src.to_string()
            + " "
            + &edge.tgt.to_string()
            + " "
            + &edge.dist.to_string()
            + "\n";
    }

    let mut f = File::create("graph.fmi").expect("Unable to create file");
    f.write_all(data.as_bytes()).expect("unable to write file");

}
