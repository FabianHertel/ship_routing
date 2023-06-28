use std::{time::SystemTime, fs::{self, File}, error::Error, io::{Write}};
use rayon::{prelude::*};
use rand::Rng;

use graph_lib::{Coordinates, Node, Edge};

use crate::island::{Island, min_distance, GRID_DIVISIONS, grid_cell_of_coordinate, GridCell};

pub const MOST_SOUTHERN_LAT_IN_SEA: f32 = -78.02;

pub fn generate_map(filename_out: &str, import_prefix: &str) -> Result<(), Box<dyn Error>> {

    println!("1/?: Read GeoJSONs parallel ...");
    let now = SystemTime::now();
    let coastlines: Vec<Vec<Vec<f32>>> = read_geojsons(import_prefix);
    println!("1/?: Finished in {} sek", now.elapsed().unwrap().as_secs());

    println!("2/?: Precalculations for islands/continents ...");
    let now = SystemTime::now();
    let mut island_grid: Vec<Vec<GridCell>> = GRID_DIVISIONS.iter().map(|e| vec![GridCell::WATER; *e]).collect();
    let islands: Vec<Island> = coastlines.iter().map(|e| Island::new(e.to_owned())).collect();
    islands.iter().for_each(|island| island.add_to_grid(&mut island_grid));
    println!("2/?: Finished precalculations in {} sek", now.elapsed().unwrap().as_secs());

    generate_graph_on_sphere(&island_grid, 1000000, filename_out);

    Ok(())
}

/**
 * read continents and islands from geojsons and sort it from big to small
 */
pub fn read_geojsons(prefix: &str) -> Vec<Vec<Vec<f32>>> {
    let mut coastlines: Vec<Vec<Vec<f32>>> =  ["continents", "big_islands", "islands", "small_islands"]
        .par_iter()
        .map(|filename| {
            let now = SystemTime::now();
            let filepath = format!("./data/geojson/{}.json", prefix.to_owned() + "_" + filename);
            let geojson_str = fs::read_to_string(&filepath)
                .expect(&format!("Unable to read JSON file {}", &filepath));
            let geojson_str = &geojson_str[18..];
            let coastlines_part: Vec<Vec<Vec<f32>>> = geojson_str.split("[[").into_iter().map(|island_str| {
                if island_str.len() > 5 {
                    island_str.split('[').into_iter().map(|coordinates| {
                        let mut coordinates_split = coordinates.split(&[',', ']', ' '][..]);
                        let mut coordinates = vec![];
                        while coordinates.len() < 2 {
                            let number = coordinates_split.nth(0);
                            if number != Some("") {
                                coordinates.push(number.unwrap().parse::<f32>().unwrap())
                            }
                        }
                        return coordinates;
                    }).collect()
                } else {
                    vec![]
                }
            }).collect();
            println!(
                "Parsing {} finished after {} sek",
                filepath,
                now.elapsed().unwrap().as_secs()
            );
            return coastlines_part;
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
pub fn point_on_land_test(lon: f32, lat: f32, island_grid: &Vec<Vec<GridCell>>) -> bool {
    // no point in water is more south than -78.02
    if lat < MOST_SOUTHERN_LAT_IN_SEA {return true}

    let [grid_row, cell_in_row] = grid_cell_of_coordinate(lon, lat);

    // println!("(lon, lat): {},{} makes {},{}", lon, lat, grid_row, cell_in_row);
    match island_grid[grid_row][cell_in_row] {
        GridCell::WATER => false,
        GridCell::LAND(_) => true,
        GridCell::ISLANDS(ref islands) => {
            for island in islands {
                if point_in_polygon_test(lon, lat, island) {
                    return true;
                }
            }
            return false;
        },
    }
}

#[inline]
pub fn point_in_polygon_test(lon: f32, lat: f32, island: &Island) -> bool {
    // println!("Island in cell: {}", island);
    let in_bounding_box = lon > island.get_bounding_box()[0][0]
    && lon < island.get_bounding_box()[0][1]
    && lat > island.get_bounding_box()[1][0]
    && lat < island.get_bounding_box()[1][1];
    if in_bounding_box {
        let in_range_of_ref_points =
            min_distance(&Coordinates(lon, lat), &island.get_reference_points()).0 < *island.get_max_dist_from_ref();
        if in_range_of_ref_points {
            // println!("Island center: {}; max_dist_from_ref: {}; point distance: {}, coastline_points: {}", island, island.get_max_dist_from_ref(), &island.get_reference_points()[0].distance_to(&Coordinates(lon, lat)), island.get_coastline().len());
            let mut in_water = false;
            let polygon = &island.get_coastline();
            if island.get_lon_distribution().len() > 0 {
                let index_in_lon_distr = ((lon - island.get_bounding_box()[0][0])
                    / island.get_lon_distribution_distance())
                    .floor() as usize;
                let mut last_point_i: usize = 0;
                // println!("Checking {} edges", &island.get_lon_distribution()[index_in_lon_distr].len());
                for point_i in &island.get_lon_distribution()[index_in_lon_distr] {
                    if *point_i != last_point_i + 1 && *point_i > 0 {
                        // check edge before only if not already checked
                        let (start, end) = (&polygon[point_i - 1], &polygon[*point_i]);
                        if line_cross_check(start, end, lon, lat) {in_water = !in_water}
                    }
                    // check edge after always if point is not the last one
                    if *point_i < island.get_coastline().len() - 1 {
                        let (start, end) = (&polygon[*point_i], &polygon[point_i + 1]);
                        if line_cross_check(start, end, lon, lat) {in_water = !in_water}
                    }
                    last_point_i = *point_i;
                }
            } else {
                // println!("No lon distibution: Checking {} edges", &island.get_coastline().len());
                for j in 1..polygon.len() {
                    // ignore first point in polygon, because first and last will be the same
                    let (start, end) = (&polygon[j - 1], &polygon[j]);
                    if line_cross_check(start, end, lon, lat) {in_water = !in_water}
                }
            }
            if in_water {
                return true;
            }
        } else {
            //println!("Ref points saved checking {} edges of this continent: {}, {}!!!", island.coastline.len(), island.reference_points[0].0, island.reference_points[0].1);
        }
    }
    return false;
}

/**
 * checks if point with lon lat crosses edge between start and end by going north
 */
#[inline]
fn line_cross_check(start: &Coordinates, end: &Coordinates, lon: f32, lat: f32) -> bool {
    if (start.0 > lon) != (end.0 > lon) {
        // check if given lon of point is between start and end point of edge
        if (start.1 > lat) && (end.1 > lat) {
            // if both start and end point are north, the going north will cross
            // println!("Line crossed: {}; {}", start, end);
            return true;
        } else if (start.1 > lat) || (end.1 > lat) {
            // if one of start and end point are south, we have to check... (happens rarely for coastline)
            let slope = (lat - start.1) * (end.0 - start.0)
                - (end.1 - start.1) * (lon - start.0);
            if (slope < 0.0) != (end.0 < start.0) {
                // println!("Line crossed (rare case!)");
                return true;
            }
        }
    }
    return false;
}

fn generate_graph_on_sphere(island_grid: &Vec<Vec<GridCell>>, number_of_points: u32, filename_out: &str) -> () {
    let mut rng = rand::thread_rng();
    let mut new_node: Node;
    let mut points: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut lat: f32;
    let mut lon: f32;
    let mut norm: f32;
    let mut id = 0;
    let mut counter = 0;
    let mut grid: Vec<Vec<Vec<Node>>> = vec![vec![Vec::new(); 180]; 360];
    let mut data = String::new();
    let max_distance = 30.0;

    let now = SystemTime::now();
    while counter < number_of_points {
        (lon, lat, norm) = random_point_on_sphere(&mut rng);

        if norm <= 1.0 {
            if !point_on_land_test(lon, lat, island_grid) {
                new_node = Node {
                    id: 0,
                    lat: lat,
                    lon: lon,
                };

                grid[(lon.round() + 180.0 - 1.0) as usize][(lat.round() + 90.0 - 1.0) as usize]
                    .push(new_node);
                counter = counter + 1;
                if counter % 100000 == 0 {println!("Generated {} points", counter)}
                //print!("[{},{}],", lon, lat);
            }
        }
    }
    println!("Finished generating points in {} sek", now.elapsed().unwrap().as_secs());

    let now = SystemTime::now();
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

    print!("Connected lat colomns: ");
    for i in 0..360 {
        for j in 0..180 {
            if grid[i][j].len() >= 1 {
                let mut closest_ne = 0;
                let mut distance_ne = 40000.0;
                let mut closest_se = 0;
                let mut distance_se = 40000.0;
                let mut temp_nodes: Vec<Node> = Vec::new();
                let mut distance_to_node;

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
        print!("{}-{},", i, i+1)
    }
    println!("");
    println!("Finished graph connecting in {} sek", now.elapsed().unwrap().as_secs());

    let now = SystemTime::now();
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
    println!("Finished file write {} sek", now.elapsed().unwrap().as_secs());

}

#[inline]
pub fn random_point_on_sphere<R: Rng + ?Sized>(rng: &mut R) -> (f32, f32, f32) {
    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;
    let mut z: f32 = 0.0;

    while ((x * x + y * y + z * z).sqrt()) < 0.001 {
        x = rng.gen_range(-1.0..1.0);
        y = rng.gen_range(-1.0..1.0);
        z = rng.gen_range(-1.0..1.0);
    }
    let norm = (x * x + y * y + z * z).sqrt();

    x = x / norm;
    y = y / norm;
    z = z / norm;

    let lat = z.asin().to_degrees(); // asin(Z/R)
    let lon = y.atan2(x).to_degrees(); // atan2(y,x)

    return (lon, lat, norm);
}