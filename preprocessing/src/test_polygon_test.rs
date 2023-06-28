use std::{time::SystemTime};

use rand::{rngs::StdRng, SeedableRng};

use crate::{generate_map::{read_geojsons, point_in_polygon_test, random_point_on_sphere}, island::{Island, GRID_DIVISIONS}};

pub fn static_polygon_tests(import_prefix: &str) {

    println!("1/?: Read GeoJSONs parallel ...");
    let now = SystemTime::now();
    let coastlines: Vec<Vec<Vec<f32>>> = read_geojsons(import_prefix);
    println!("1/?: Finished in {} sek", now.elapsed().unwrap().as_secs());


    println!("2/?: Precalculations for islands/continents ...");
    let now = SystemTime::now();
    let islands: Vec<Island> = coastlines.iter().map(|e| Island::new(e.to_owned())).collect();
    let mut island_grid: Vec<Vec<Vec<&Island>>> = GRID_DIVISIONS.iter().map(|e| vec![vec![]; *e]).collect();
    islands.iter().for_each(|island| island.add_to_grid(&mut island_grid));
    println!("2/?: Finished precalculations in {} sek", now.elapsed().unwrap().as_secs());
    
    // unmeasured test beforehand to fill caches or similar
    point_in_polygon_test(0.0, 0.0, &island_grid);
    let now = SystemTime::now();
    println!("Point on land (Atlantic): {}", point_in_polygon_test(0.0, 0.0, &island_grid)); // Atlantic
    println!("Finished test in {} micros", now.elapsed().unwrap().as_micros());
    let now = SystemTime::now();
    println!("Point on land (US): {}", point_in_polygon_test(-104.2758092369033, 34.117786526143604, &island_grid)); //US
    println!("Finished test in {} micros", now.elapsed().unwrap().as_micros());
    let now = SystemTime::now();
    println!("Point on land (Arktis): {}", point_in_polygon_test(-27.24044854389621, 70.01752410356319, &island_grid)); // North of Grönland
    println!("Finished test in {} micros", now.elapsed().unwrap().as_micros());
    let now = SystemTime::now();
    println!("Point on land (Antarctica): {}", point_in_polygon_test(71.55, -74.1878186, &island_grid)); //Antarctica
    println!("Finished test in {} micros", now.elapsed().unwrap().as_micros());
    let now = SystemTime::now();
    println!("Point on land (Mid of pacific): {}", point_in_polygon_test(-144.1294183409396, -47.75776979131451, &island_grid));
    println!("Finished test in {} micros", now.elapsed().unwrap().as_micros());
    let now = SystemTime::now();
    println!("Point on land (Russia): {}", point_in_polygon_test(82.35471714457248, 52.4548566256728, &island_grid));
    println!("Finished test in {} micros", now.elapsed().unwrap().as_micros());

    let now = SystemTime::now();
    println!("Testing 1000 random points...");

    let mut rng: StdRng = StdRng::seed_from_u64(0);     // seed in contrast to real to have comparable conditions
    let mut lat: f32;
    let mut lon: f32;
    let mut norm: f32;
    let mut counter = 0;
    let mut slow_points: Vec<(u128, f32, f32, i32)> = vec![];
    let mut water_coordinates: Vec<[f32; 2]> = vec![];

    while counter < 1000 {
        (lon, lat, norm) = random_point_on_sphere(&mut rng);

        if norm <= 1.0 {
            let now = SystemTime::now();
            if !point_in_polygon_test(lon, lat, &island_grid) {
                water_coordinates.push([lon as f32, lat as f32]);
            }
            let elapsed_time = now.elapsed().unwrap().as_millis();
            if elapsed_time > 2 {slow_points.push((elapsed_time, lon, lat, counter));}
            counter += 1;
        }
    }
    println!("Finished 1000 points in {} millis", now.elapsed().unwrap().as_millis());
    println!("{:?}", water_coordinates);

    for (elapsed_time, lon, lat, counter) in &slow_points {
        println!("{} millis for {},{} with index {}", elapsed_time, lon, lat, counter);
    }
    if slow_points.len() > 0 {
        println!("Slow points as list: {}", slow_points.iter().map(|(_, lon, lat, _)| format!("[{lon},{lat}]")).reduce(|e, f| e + "," + &f).unwrap());
    }
}