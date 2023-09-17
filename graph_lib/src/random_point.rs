use rand::Rng;
use crate::{island::{Island, grid_cell_of_coordinate, GridCell, MOST_SOUTHERN_LAT_IN_SEA}, Coordinates};

/**
 * generates a random point in water, uniformly distributed
 */
#[inline]
pub fn random_point_in_water<R: Rng + ?Sized>(rng: &mut R, island_grid: &Vec<Vec<GridCell>>) -> (f32, f32) {
    let mut point = random_point_on_sphere(rng);
    while point_on_land_test(point.0, point.1, island_grid) {
        point = random_point_on_sphere(rng);
    }
    return point;
}


/**
 * generates a random 3D vector, returns in lat, lon and length
 */
#[inline]
pub fn random_point_on_sphere<R: Rng + ?Sized>(rng: &mut R) -> (f32, f32) {
    let mut norm = 2.0;
    let mut lat = 0.0;
    let mut lon = 0.0;
    // if its length (=norm) <= 1, then it's in the earth ball; if not, we ignore the point
    // so we have only points which are uniformly distributed vectors in the volume of the earth here
    // we ignore no the length, so map all these vectors to length 1, which means to the earth surface
    while norm > 1.0 {
            
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        let mut z: f32 = 0.0;

        while ((x * x + y * y + z * z).sqrt()) < 0.001 {
            x = rng.gen_range(-1.0..1.0);
            y = rng.gen_range(-1.0..1.0);
            z = rng.gen_range(-1.0..1.0);
        }
        norm = (x * x + y * y + z * z).sqrt();

        x = x / norm;
        y = y / norm;
        z = z / norm;

        lon = y.atan2(x).to_degrees(); // atan2(y,x)
        lat = z.asin().to_degrees(); // asin(Z/R)

    }
    return (lon, lat);
}


/**
 * This method will check, the given point is on land or inside water.
 * If on land, true will be returned.
 * This will be done by checking how many coastlines will be crossed going to the northpole.
 * We consider the earth as a 2D map with (x,y) = (lon, lat).
 * So the northpole has the same width as the equator, which is fine for this algorithm since coastlines are short.
 * According to our measurements of the coastlines with the table from https://dataverse.jpl.nasa.gov/dataset.xhtml?persistentId=hdl:2014/41271 there is a maximum error of single meters.
 * From the given point we check how many coastlines are crossed going straight north.
 * If it is even, we are in the sea. If odd, we are on land.
 * Note: Antartica avoids -180 to 180 edge, so coastline goes to the southpole and around it.
 */
pub(crate) fn point_on_land_test(lon: f32, lat: f32, island_grid: &Vec<Vec<GridCell>>) -> bool {
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
pub(crate) fn point_in_polygon_test(lon: f32, lat: f32, island: &Island) -> bool {
    // println!("Island in cell: {}", island);
    let in_bounding_box = lon > island.get_bounding_box()[0][0]
    && lon < island.get_bounding_box()[0][1]
    && lat > island.get_bounding_box()[1][0]
    && lat < island.get_bounding_box()[1][1];
    if in_bounding_box {
        // println!("Island center: {}; max_dist_from_ref: {}; point distance: {}, coastline_points: {}", island, island.get_max_dist_from_ref(), &island.get_reference_points()[0].distance_to(&Coordinates(lon, lat)), island.get_coastline().len());
        let mut in_water = false;
        let polygon = &island.get_coastline();
        if island.get_lon_distribution().len() > 0 {
            let index_in_lon_distr = ((lon - island.get_bounding_box()[0][0])
                / island.get_lon_distribution_distance())
                .floor() as usize;
            let mut last_point_i: usize = 0;
            // println!("Checking {} edges", &island.get_lon_distribution()[index_in_lon_distr].len());

            // check only edges which have one end in the same vertical layer
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