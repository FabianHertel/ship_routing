use graph_lib::Coordinates;
use lombok::Getter;

use crate::generate_map::MOST_SOUTHERN_LAT_IN_SEA;

// grid order: from north to south, from east to west
// so always from high coordinate values to small
pub const GRID_DIVISIONS: [usize; 36] = [3,9,16,22,28,33,39,44,49,53,57,61,64,67,69,70,71,72,72,71,70,69,67,64,61,57,53,49,44,39,33,28,22,16,9,3];
const GRID_DISTANCE: f64 = 180.0 / GRID_DIVISIONS.len() as f64;

/**
 * bounding box: [[min_lon, max_lon], [min_lat, max_lat]]
 */
#[derive(Getter)]
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

impl std::fmt::Debug for Island {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Islandcenter: ({})", self.reference_points[0])
    }
}

impl Island {
    pub fn new(coastline: Vec<Vec<f64>>) -> Island {
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
                && coastline[i][1] > MOST_SOUTHERN_LAT_IN_SEA
                // around southpole are edges with big jumps, but can be ignored; fix to distribute antarctica
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

    pub fn add_to_grid<'a>(&'a self, island_grid: &mut Vec<Vec<Vec<&'a Island>>>) {
        // fill grid
        for (lat_index, lat_row_count) in GRID_DIVISIONS.iter().enumerate() {
            // check if boundingbox inside latitude row
            // lower bounding box edge under upper line of lat row          && upper bounding box edge over lower line of lat_row
            if self.bounding_box[1][0] < 90.0 - lat_index as f64 * GRID_DISTANCE && self.bounding_box[1][1] > 90.0 - (lat_index+1) as f64 * GRID_DISTANCE {
                for cell_in_row in 0..*lat_row_count {
                    let lon_dist = 360.0 / *lat_row_count as f64;
                    // west box edge western of eastern border of grid                 && east box edge easter of western border of grid
                    if self.bounding_box[0][0] < 180.0 - cell_in_row as f64 * lon_dist && self.bounding_box[0][1] > 180.0 - (cell_in_row+1) as f64 * lon_dist {
                        island_grid[lat_index][cell_in_row].push(&self);
                    }
                }
            }
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


pub fn min_distance(x: &Coordinates, reference_points: &Vec<Coordinates>) -> (f64, Coordinates) {
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