use std::{collections::HashSet, time::SystemTime, fs};
use lombok::Getter;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use crate::{Coordinates, random_point::point_in_polygon_test};

pub const MOST_SOUTHERN_LAT_IN_SEA: f32 = -78.02;

// grid order: from north to south, from east to west
// so always from high coordinate values to small
pub const GRID_DIVISIONS: [usize; 36] = [3,9,16,22,28,33,39,44,49,53,57,61,64,67,69,70,71,72,72,71,70,69,67,64,61,57,53,49,44,39,33,28,22,16,9,3];
const GRID_DISTANCE: f32 = 180.0 / GRID_DIVISIONS.len() as f32;

/**
 * enum to define if a grid cell is completely filled with water, completely filled with land or contains water and land
 */
#[derive(Clone)]
pub enum GridCell<'a> {
    WATER,
    LAND(&'a Island),
    ISLANDS(Vec<&'a Island>),
}

impl std::fmt::Debug for GridCell<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GridCell::LAND(_) => write!(f, "LAND"),
            GridCell::WATER => write!(f, "WATER"),
            GridCell::ISLANDS(islands) => write!(f, "ISLANDS({})", islands.len()),
        }
    }
}

#[inline]
pub fn grid_cell_of_coordinate(lon: f32, lat: f32) -> [usize; 2] {
    // northpole is row 0, southpole is last row
    // not using -90 and -180 to avoid index out of bounds in our grid
    let grid_row = (-(lat - 89.999) * GRID_DIVISIONS.len() as f32 / 180.0) as usize;
    let cell_in_row = (-(lon - 179.999) * GRID_DIVISIONS[grid_row] as f32 / 360.0) as usize;
    return [grid_row, cell_in_row];
}

/**
 * bounding box: [[min_lon, max_lon], [min_lat, max_lat]]
 */
#[derive(Getter)]
pub struct Island {
    coastline: Vec<Coordinates>,
    bounding_box: [[f32; 2]; 2],
    center: Coordinates,
    lon_distribution: Vec<Vec<usize>>,
    lon_distribution_distance: f32,
    grid_cells_touched: Vec<HashSet<usize>>
}

impl std::fmt::Display for Island {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Islandcenter: ({})", self.center)
    }
}

impl std::fmt::Debug for Island {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Islandcenter: ({})", self.center)
    }
}

impl Island {
    pub fn new(coastline: Vec<Vec<f32>>) -> Island {
        let mut bounding_box = [[180.0, -180.0], [90.0, -90.0]];
        let mut coastline_formatted: Vec<Coordinates> = vec![];
        let mut max_lon_jump = 0.0;
        let mut grid_cells_touched: Vec<HashSet<usize>> = vec![HashSet::new(); GRID_DIVISIONS.len()];
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
            if i < coastline.len() - 1
                && (coastline[i + 1][0] - coastline[i][0]).abs() > max_lon_jump
                && coastline[i][1] > MOST_SOUTHERN_LAT_IN_SEA
                // around southpole are edges with big jumps, but can be ignored; fix to distribute antarctica
            {
                max_lon_jump = (coastline[i + 1][0] - coastline[i][0]).abs();
            }
            coastline_formatted.push(Coordinates::from_vec(&coastline[i]));

            let grid_cell = grid_cell_of_coordinate(coastline[i][0], coastline[i][1]);
            // println!("{:?} for point {},{}", grid_cell, coastline[i][0], coastline[i][1]);
            grid_cells_touched[grid_cell[0]].insert(grid_cell[1]);
        }
        const MIN_LON_DISTR_DIFF: f32 = 0.2;
        let lon_distribution_distance = max(max_lon_jump, MIN_LON_DISTR_DIFF);
        let center = Coordinates((bounding_box[0][1] - bounding_box[0][0]) / 2.0, (bounding_box[1][1] - bounding_box[1][0]) / 2.0);


        let mut lon_distribution: Vec<Vec<usize>> = vec![];
        const MIN_SIZE_FOR_LON_DISTR: usize = 1000;
        if coastline.len() > MIN_SIZE_FOR_LON_DISTR {
            // println!("max: {}, min: {}, lon_dist: {}", bounding_box[0][1], bounding_box[0][0], lon_distribution_distance);
            if (bounding_box[0][1] - bounding_box[0][0]) > 10.0 * lon_distribution_distance {
                let n_seperations = ((bounding_box[0][1] - bounding_box[0][0])
                    / lon_distribution_distance)
                    .ceil() as usize;
                // println!("Max jump: {}; distr size: {}", max_lon_jump, n_seperations);
                lon_distribution = vec![vec![]; n_seperations];
            }
        }

        for i in 0..coastline_formatted.len() {
            if coastline.len() > MIN_SIZE_FOR_LON_DISTR
                && (bounding_box[0][1] - bounding_box[0][0]) > 10.0 * lon_distribution_distance
            {
                let index_in_lon_distr = ((coastline_formatted[i].0 - bounding_box[0][0])
                    / lon_distribution_distance)
                    .floor() as usize;
                lon_distribution[index_in_lon_distr].push(i);
            }
        }

        Island {
            coastline: coastline_formatted,
            bounding_box,
            center,
            lon_distribution,
            lon_distribution_distance,
            grid_cells_touched,
        }
    }

    pub fn add_to_grid<'a>(&'a self, island_grid: &mut Vec<Vec<GridCell<'a>>>) {
        for (row_index, grid_row) in self.grid_cells_touched.iter().enumerate() {
            if grid_row.len() > 0 {
                let (mut max, mut min) = (0 as usize, usize::MAX);
                for cell_in_row in grid_row {
                    // add itself to all touched grid cells
                    if *cell_in_row >= island_grid[row_index].len() {
                        println!("{:?} for row {}, row: {:?}, island: {:?}", island_grid, row_index, grid_row, self);
                    }
                    match island_grid[row_index][*cell_in_row] {
                        GridCell::WATER => island_grid[row_index][*cell_in_row] = GridCell::ISLANDS(vec![&self]),
                        GridCell::LAND(_) => (), // occurs only for polygons in polygons, so islands in the caspian sea
                        GridCell::ISLANDS(ref mut islands) => islands.push(&self)
                    }
                    max = usize::max(max, *cell_in_row);
                    min = usize::min(min, *cell_in_row);
                }
                
                // check if cells are completely in polygon...
                for cell in min+1..max {
                    if !grid_row.contains(&cell) {
                        let lon_center = 180.0 - cell as f32 * 360.0 / GRID_DIVISIONS[row_index] as f32;
                        let lat_center = 90.0 - row_index as f32 * GRID_DISTANCE;
                        if point_in_polygon_test(lon_center, lat_center, &self) {
                            island_grid[row_index][cell] = GridCell::LAND(&self);
                        }
                    }
                }
            }
        }
    }
}

fn max(v1: f32, v2: f32) -> f32 {
    if v1 > v2 {
        return v1;
    } else {
        return v2;
    }
}


/**
 * read continents and islands from geojsons in parallel and sort it from big to small;
 * islands splitted in different files to speedup reading and writing
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