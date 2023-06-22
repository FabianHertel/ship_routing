use std::{time::SystemTime, fs::{self, File}, error::Error, io::{Write}};
use geojson::{GeoJson, Geometry, Value};
use rayon::{prelude::*};
use rand::Rng;

use graph_lib::{Coordinates, Node, Edge};

use crate::island::{Island, min_distance, GRID_DIVISIONS};

pub fn generate_map(filename_out: &str) -> Result<(), Box<dyn Error>> {

    println!("1/?: Read GeoJSONs parallel ...");
    let now = SystemTime::now();
    let coastlines: Vec<Vec<Vec<f64>>> = read_geojsons("reduced");
    println!("1/?: Finished in {} sek", now.elapsed().unwrap().as_secs());

    println!("2/?: Precalculations for islands/continents ...");
    let now = SystemTime::now();
    let islands: Vec<Island> = coastlines.iter().map(|e| Island::new(e.to_owned())).collect();
    let mut island_grid: Vec<Vec<Vec<&Island>>> = GRID_DIVISIONS.iter().map(|e| vec![vec![]; *e]).collect();
    islands.iter().for_each(|island| island.add_to_grid(&mut island_grid));
    println!("2/?: Finished precalculations in {} sek", now.elapsed().unwrap().as_secs());

    random_points_on_sphere(&island_grid, 1000000, filename_out);

    Ok(())
}

/**
 * read continents and islands from geojsons and sort it from big to small
 */
pub fn read_geojsons(prefix: &str) -> Vec<Vec<Vec<f64>>> {
    let mut coastlines =  ["continents", "big_islands", "islands", "small_islands"]
        .par_iter()
        .map(|filename| {
            let now = SystemTime::now();
            let filepath = format!("./data/geojson/{}.json", prefix.to_owned() + "_" + filename);
            let geojson_str = fs::read_to_string(&filepath)
                .expect(&format!("Unable to read JSON file {}", &filepath));
            let geojson: GeoJson = geojson_str.parse::<GeoJson>().unwrap(); // needs much of time (4-5min for world)
            println!(
                "Parsing {} finished after {} sek",
                filepath,
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

        coastlines.sort_by(|a, b| b.len().cmp(&a.len()));
        return coastlines;
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
pub fn point_in_polygon_test(lon: f64, lat: f64, island_grid: &Vec<Vec<Vec<&Island>>>) -> bool {
    // in case lon or lat is exactly on the edge we can get an index out of bounds
    let (mut lon, mut lat) = (lon, lat);
    if lon == -180.0 {lon -= 0.0001};
    if lat == -90.0 {lat -= 0.0001};

    // northpole is row 0, southpole is last row
    let grid_row = (-(lat - 90.0) * GRID_DIVISIONS.len() as f64 / 180.0) as usize;
    let cell_in_row = (-(lon - 180.0) * GRID_DIVISIONS[grid_row] as f64 / 360.0) as usize;
    // println!("(lon, lat): {},{} makes {},{}", lon, lat, grid_row, cell_in_row);
    for island in &island_grid[grid_row][cell_in_row] {
        // println!("Island in cell: {}", island);
        let in_bounding_box = lon > island.get_bounding_box()[0][0]
            && lon < island.get_bounding_box()[0][1]
            && lat > island.get_bounding_box()[1][0]
            && lat < island.get_bounding_box()[1][1];
        if in_bounding_box {
            let in_range_of_ref_points =
                min_distance(&Coordinates(lon, lat), &island.get_reference_points()).0
                    < *island.get_max_dist_from_ref();
            if in_range_of_ref_points {
                // println!("Island center: {}; max_dist_from_ref: {}; point distance: {}, coastline_points: {}", island, island.max_dist_from_ref, distance_between(&island.reference_points[0], &Coordinates(lon, lat)), island.coastline.len());
                let mut in_water = false;
                let polygon = &island.get_coastline();
                if island.get_lon_distribution().len() > 0 {
                    let index_in_lon_distr = ((lon - island.get_bounding_box()[0][0])
                        / island.get_lon_distribution_distance())
                        .floor() as usize;
                    let mut last_point_i: usize = 0;
                    for point_i in &island.get_lon_distribution()[index_in_lon_distr] {
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
                        if *point_i < island.get_coastline().len() - 1 {
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

pub fn random_points_on_sphere(island_grid: &Vec<Vec<Vec<&Island>>>, number_of_points: u32, filename_out: &str) -> () {
    let mut rng = rand::thread_rng();
    let mut new_node: Node;
    let mut points: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut x: f64;
    let mut y: f64;
    let mut z: f64;
    let mut lat: f64;
    let mut lon: f64;
    let mut norm: f64;
    let mut id = 0;
    let mut counter = 0;
    let mut grid: Vec<Vec<Vec<Node>>> = vec![vec![Vec::new(); 180]; 360];
    let mut data = String::new();
    let max_distance = 30.0;

    while counter < number_of_points {
        x = 0.0;
        y = 0.0;
        z = 0.0;

        while ((x * x + y * y + z * z).sqrt()) < 0.001 {
            x = rng.gen_range(-1.0..1.0);
            y = rng.gen_range(-1.0..1.0);
            z = rng.gen_range(-1.0..1.0);
        }
        norm = (x * x + y * y + z * z).sqrt();

        x = x / norm;
        y = y / norm;
        z = z / norm;

        lat = z.asin().to_degrees(); // asin(Z/R)
        lon = y.atan2(x).to_degrees(); // atan2(y,x)

        if norm <= 1.0 {
            if !point_in_polygon_test(lon, lat, island_grid) {
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

    let mut f = File::create("data/".to_owned() + filename_out).expect("Unable to create file");
    f.write_all(data.as_bytes()).expect("unable to write file");

}
