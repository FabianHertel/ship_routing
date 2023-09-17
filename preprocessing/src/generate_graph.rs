use std::{time::SystemTime, error::Error, io::{Write, stdout, self}, collections::HashMap};

use graph_lib::{Coordinates, Node, Edge, file_interface::print_graph_to_file, distance_between, island::{GridCell, read_geojsons, GRID_DIVISIONS, Island}, random_point::random_point_in_water};


pub fn generate_graph(filename_out: &str, import_prefix: &str) -> Result<(), Box<dyn Error>> {
    const NUMBER_OF_NODES: u32 = 4000000;

    println!("1/5: Read GeoJSONs parallel ...");
    let now = SystemTime::now();
    let coastlines: Vec<Vec<Vec<f32>>> = read_geojsons(import_prefix);
    println!("1/5: Finished: Imported {} coastlines in {} sek", coastlines.len(), now.elapsed().unwrap().as_secs());

    println!("2/5: Precalculations for islands/continents ...");
    let now = SystemTime::now();
    let mut island_grid: Vec<Vec<GridCell>> = GRID_DIVISIONS.iter().map(|e| vec![GridCell::WATER; *e]).collect();
    let islands: Vec<Island> = coastlines.iter().map(|e| Island::new(e.to_owned())).collect();
    islands.iter().for_each(|island| island.add_to_grid(&mut island_grid));
    let n_grid_cells = GRID_DIVISIONS.into_iter().reduce(|e, f| e + f).unwrap();
    println!("2/5: Finished precalculations of islands and put into {} grid cells in {} sek", n_grid_cells, now.elapsed().unwrap().as_secs());

    println!("3/5: Generating {} random points in ocean ...", NUMBER_OF_NODES);
    let now = SystemTime::now();
    let graph_grid = generate_random_points_in_ocean(&island_grid, NUMBER_OF_NODES);
    println!("3/5 Finished generating points in {} min", now.elapsed().unwrap().as_secs() as f32 / 60.0);

    println!("4/5: Connecting graph ...");
    let now = SystemTime::now();
    let (nodes, edges) = connect_graph(graph_grid);
    println!("4/5 Finished graph creating {} edges in {} min", edges.len(), now.elapsed().unwrap().as_secs() as f32 / 60.0);

    println!("5/5: Writing graph into {} ...", filename_out);
    let now = SystemTime::now();
    print_graph_to_file(&nodes, &edges, filename_out);
    println!("5/5 Finished file write {} sek", now.elapsed().unwrap().as_secs());
    Ok(())
}

fn generate_random_points_in_ocean(island_grid: &Vec<Vec<GridCell>>, number_of_points: u32) -> Vec<Vec<Vec<Node>>> {
    let mut grid: Vec<Vec<Vec<Node>>> = vec![vec![Vec::new(); 180]; 360];
    let mut lat: f32;
    let mut lon: f32;
    let mut new_node: Node;
    let mut rng = rand::thread_rng();
    let mut counter = 0;

    while counter < number_of_points {
        // first we generate just a uniformly distributed random 3D vectors
        (lon, lat) = random_point_in_water(&mut rng, island_grid);

        new_node = Node { id: 0, lat, lon };
        grid[(lon + 179.99).floor() as usize][(lat + 89.99).floor() as usize]
            .push(new_node);
        counter = counter + 1;
        if counter % 1000 == 0 {
            print!("\rGenerating... {}/{} points", counter, number_of_points);
            stdout().flush().unwrap();
        }
        //print!("[{},{}],", lon, lat);
    }
    println!("");
    
    return grid;
}


fn connect_graph(mut graph_grid: Vec<Vec<Vec<Node>>>) -> (Vec<Node>, Vec<Edge>) {
    let mut points: Vec<Node> = Vec::new();
    let mut edges_set = HashMap::new();
    let mut final_edges: Vec<Edge> = Vec::new();
    let mut id = 0;
    let max_distance = 30000;

    // set ids according to the grid, so from west to east and then from north to south 
    for i in 0..360 {
        for j in 0..180 {
            for k in 0..graph_grid[i][j].len() {
                graph_grid[i][j][k].id = id;
                points.push(graph_grid[i][j][k].clone());
                id = id + 1;
            }
        }
    }

    // search for closest north-east, north-west, south-east and south-west neighbours and connect them
    for i in 0..360 {
        for j in 0..180 {
            for k in &graph_grid[i][j] {
                // calclate how many grids to the right/left shold be considered
                let distance_to_right_end = k.distance_to(&Coordinates(i as f32 - 179.0, k.lat));
                let distance_to_left_end = k.distance_to(&Coordinates(i as f32 - 180.0, k.lat));
                let distance_to_north_end = k.distance_to(&Coordinates(k.lon, j as f32 - 90.0));
                let distance_to_south_end = k.distance_to(&Coordinates(k.lon, j as f32 - 89.0));
                let grid_with = distance_between(i as f32 - 179.0, k.lat, i as f32 - 180.0, k.lat);
                
                let grids_right = (((max_distance as f32 - distance_to_right_end) / grid_with).ceil() as usize).min(180); 
                let grids_left = (((max_distance as f32 - distance_to_left_end) / grid_with).ceil() as usize).min(180);
                let check_north = distance_to_north_end < max_distance as f32 && j > 0;
                let check_south = distance_to_south_end < max_distance as f32 && j < 179;
                
                // add all possible grids to a vector to check enough but not too much
                let mut possible_neighbours: Vec<Node> = Vec::new();
                possible_neighbours.extend(&graph_grid[i][j]);
                if check_north { possible_neighbours.extend(&graph_grid[i][j - 1]) }
                if check_south { possible_neighbours.extend(&graph_grid[i][j + 1]); }

                for r in 1..(grids_right + 1) {
                    possible_neighbours.extend(&graph_grid[(i + r).rem_euclid(360)][j]);
                    if check_north { possible_neighbours.extend(&graph_grid[(i + r).rem_euclid(360)][j - 1]) }    // if grid is not the most northern
                    if check_south { possible_neighbours.extend(&graph_grid[(i + r).rem_euclid(360)][j + 1]); } // if grid is not the most southern
                    
                }
                for l in 1..(grids_left + 1) {
                    possible_neighbours.extend(&graph_grid[(360 + i - l).rem_euclid(360)][j]);
                    if check_north { possible_neighbours.extend(&graph_grid[(360 + i - l).rem_euclid(360)][j - 1]) }  // if grid is not the most northern
                    if check_south { possible_neighbours.extend(&graph_grid[(360 + i - l).rem_euclid(360)][j + 1]); }   // if grid is not the most southern
                    // println!("left: {}, {}, {}, {}, {}", grids_right, grids_left, distance_to_right_end, distance_to_left_end, grid_with);
                }

                let mut neighbour_ne = (None, max_distance);
                let mut neighbour_se = (None, max_distance);
                let mut neighbour_sw = (None, max_distance);
                let mut neighbour_nw = (None, max_distance);

                // iterate over possible grids
                for l in &possible_neighbours {
                    if k.id != l.id {
                        let distance_to_node = k.distance_to_node(l).ceil() as u32;
                        if distance_to_node < max_distance {
                            if k.lon < l.lon && k.lat < l.lat {             //NORTH EAST
                                if distance_to_node < neighbour_ne.1 {
                                    neighbour_ne = (Some(l.id), distance_to_node);
                                }
                            } else if k.lon < l.lon && k.lat > l.lat {      // SOUTH EAST
                                if distance_to_node < neighbour_se.1 {
                                    neighbour_se = (Some(l.id), distance_to_node);
                                }
                            } else if k.lon > l.lon && k.lat > l.lat {      // SOUTH WEST
                                if distance_to_node < neighbour_sw.1 {
                                    neighbour_sw = (Some(l.id), distance_to_node);
                                }
                            } else if k.lon > l.lon && k.lat < l.lat {      // NORTH WEST
                                if distance_to_node < neighbour_nw.1 {
                                    neighbour_nw = (Some(l.id), distance_to_node);
                                }
                            }
                        }
                    }
                }

                if neighbour_ne.0.is_some() {       //create Edge for north east
                    edges_set.insert((k.id, neighbour_ne.0.unwrap()), neighbour_ne.1);
                    edges_set.insert((neighbour_ne.0.unwrap(), k.id), neighbour_ne.1);
                }

                if neighbour_se.0.is_some() {       //create Edge for south east
                    edges_set.insert((k.id, neighbour_se.0.unwrap()), neighbour_se.1);
                    edges_set.insert((neighbour_se.0.unwrap(), k.id), neighbour_se.1);
                }

                if neighbour_sw.0.is_some() {       //create Edge for south west
                    edges_set.insert((k.id, neighbour_sw.0.unwrap()), neighbour_sw.1);
                    edges_set.insert((neighbour_sw.0.unwrap(), k.id), neighbour_sw.1);
                }

                if neighbour_nw.0.is_some() {       //create Edge for north west
                    edges_set.insert((k.id, neighbour_nw.0.unwrap()), neighbour_nw.1);
                    edges_set.insert((neighbour_nw.0.unwrap(), k.id), neighbour_nw.1);
                }
            }
        }
        print!("\rConnecting... {}/360 latitudes", i + 1);
        let _ = io::stdout().flush();
    }

    for ((src, tgt), dist) in edges_set {
        final_edges.push(Edge {
            src,
            tgt,
            dist,
        });
    }

    final_edges.sort_by(|a, b| a.src.cmp(&b.src));

    // println!("");
    return (points, final_edges);
}
