use std::{time::SystemTime};

use rand::{Rng, rngs::StdRng, SeedableRng};

use crate::{generate_map::{read_geojsons, point_in_polygon_test}, island::{Island, GRID_DIVISIONS}};

pub fn static_polygon_tests() {

    println!("1/?: Read GeoJSONs parallel ...");
    let now = SystemTime::now();
    let coastlines: Vec<Vec<Vec<f64>>> = read_geojsons("complete");
    println!("1/?: Finished in {} sek", now.elapsed().unwrap().as_secs());


    println!("2/?: Precalculations for islands/continents ...");
    let now = SystemTime::now();
    let islands: Vec<Island> = coastlines.iter().map(|e| Island::new(e.to_owned())).collect();
    let mut island_grid: Vec<Vec<Vec<&Island>>> = GRID_DIVISIONS.iter().map(|e| vec![vec![]; *e]).collect();
    islands.iter().for_each(|island| island.add_to_grid(&mut island_grid));
    println!("2/?: Finished precalculations in {} sek", now.elapsed().unwrap().as_secs());
    
    let now = SystemTime::now();
    println!("Point on land (Atlantic): {}", point_in_polygon_test(0.0, 0.0, &island_grid)); // Atlantic
    println!("Finished test in {} millis", now.elapsed().unwrap().as_millis());
    let now = SystemTime::now();
    println!("Point on land (US): {}", point_in_polygon_test(-104.2758092369033, 34.117786526143604, &island_grid)); //US
    println!("Finished test in {} millis", now.elapsed().unwrap().as_millis());
    let now = SystemTime::now();
    println!("Point on land (Arktis): {}", point_in_polygon_test(-27.24044854389621, 70.01752410356319, &island_grid)); // North of Grönland
    println!("Finished test in {} millis", now.elapsed().unwrap().as_millis());
    let now = SystemTime::now();
    println!("Point on land (Antarctica): {}", point_in_polygon_test(71.55, -74.1878186, &island_grid)); //Antarctica
    println!("Finished test in {} millis", now.elapsed().unwrap().as_millis());
    let now = SystemTime::now();
    println!("Point on land (Mid of pacific): {}", point_in_polygon_test(-144.1294183409396, -47.75776979131451, &island_grid));
    println!("Finished test in {} millis", now.elapsed().unwrap().as_millis());
    let now = SystemTime::now();
    println!("Point on land (Russia): {}", point_in_polygon_test(82.35471714457248, 52.4548566256728, &island_grid));
    println!("Finished fixed points test in {} millis", now.elapsed().unwrap().as_millis());

    let now = SystemTime::now();
    println!("Testing 1000 random points...");
    let mut rng = StdRng::seed_from_u64(0);     // seed in contrast to real to have comparable conditions


    let mut x: f64;
    let mut y: f64;
    let mut z: f64;
    let mut lat: f64;
    let mut lon: f64;
    let mut norm: f64;
    let mut counter = 0;

    while counter < 1000 {
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
            counter += 1;
            point_in_polygon_test(lon, lat, &island_grid);
        }
    }
    println!("Finished 1000 points in {} millis", now.elapsed().unwrap().as_millis());
}