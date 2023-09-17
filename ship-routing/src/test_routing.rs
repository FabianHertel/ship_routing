use graph_lib::{Graph, Coordinates, ShortestPathResult, random_point::random_point_in_water, island::{read_geojsons, GridCell, GRID_DIVISIONS, Island}, Node};

use crate::{dijkstra::run_dijkstra, a_star::run_a_star, bidirectional_dijkstra::run_bidirectional_dijkstra, ch::run_ch};

struct Route {
    start: Coordinates,
    end: Coordinates,
    // optimal_route_length: u32,
    // optimal_route_node_count: usize,
    description: String
}

pub fn test_random_samples(graph: &Graph, routing: fn(src: &Node, tgt: &Node, graph: &Graph) -> ShortestPathResult) {
    let mut summed_time = 0;
    let number_of_tests = 1000;
    let mut rng = rand::thread_rng();
    println!("Import coastline");
    let coastlines = read_geojsons("complete");
    println!("Precalulations of islands");
    let islands: Vec<Island> = coastlines.iter().map(|e| Island::new(e.to_owned())).collect();
    let mut island_grid: Vec<Vec<GridCell>> = GRID_DIVISIONS.iter().map(|e| vec![GridCell::WATER; *e]).collect();
    islands.iter().for_each(|island| island.add_to_grid(&mut island_grid));

    println!("Start tests");
    for i in 0..number_of_tests {
        let (lon, lat) = &random_point_in_water(&mut rng, &island_grid);
        let start_node = graph.closest_node(&Coordinates(*lon, *lat));
        let (lon, lat) = &random_point_in_water(&mut rng, &island_grid);
        let end_node = graph.closest_node(&Coordinates(*lon, *lat));
        let result = routing(start_node, end_node, &graph);
        println!("Finished {}th query in {} ms", i, result.calculation_time);
        summed_time += result.calculation_time;
    }
    println!("Performed {} queries in average in {} ms", number_of_tests, summed_time / number_of_tests);
}

pub fn test_random_samples_compare_routings(
    graph1: &Graph, routing1: fn(src: &Node, tgt: &Node, graph: &Graph) -> ShortestPathResult,
    graph2: &Graph, routing2: fn(src: &Node, tgt: &Node, graph: &Graph) -> ShortestPathResult
) {
    let mut summed_time1 = 0;
    let mut summed_time2 = 0;
    let number_of_tests = 1000;
    let mut rng = rand::thread_rng();
    println!("Import coastline");
    let coastlines = read_geojsons("complete");
    println!("Precalulations of islands");
    let islands: Vec<Island> = coastlines.iter().map(|e| Island::new(e.to_owned())).collect();
    let mut island_grid: Vec<Vec<GridCell>> = GRID_DIVISIONS.iter().map(|e| vec![GridCell::WATER; *e]).collect();
    islands.iter().for_each(|island| island.add_to_grid(&mut island_grid));

    println!("Start tests");
    for i in 0..number_of_tests {
        let (lon, lat) = &random_point_in_water(&mut rng, &island_grid);
        let start_node = graph1.closest_node(&Coordinates(*lon, *lat));
        let (lon, lat) = &random_point_in_water(&mut rng, &island_grid);
        let end_node = graph1.closest_node(&Coordinates(*lon, *lat));
        let result1 = routing1(start_node, end_node, &graph1);
        let result2 = routing2(start_node, end_node, &graph2);
        assert_eq!(result1.distance, result2.distance);
        println!(
            "Finished {}th query: both with distance {}; routing 1 in {} ms and routing 2 in {} ms",
            result1.distance, i, result1.calculation_time, result2.calculation_time
        );
        summed_time1 += result1.calculation_time;
        summed_time2 += result2.calculation_time;
    }
    println!("Performed {} queries in averages: routint 1 in {} ms, routing 2 in {} ms", number_of_tests, summed_time1 / number_of_tests, summed_time2 / number_of_tests);
}

/**
 * Here all routings can be tested for a given graph. It will use some predefined start and end points
 * Distance DEPENDS ON THE GRAPH
 */ 
pub fn test_samples(graph: &Graph, ch_graph: &Graph) {
    let coords_med_to_red_sea = Route {
        start: Coordinates(28.677027, 33.622692),
        end: Coordinates(37.114494, 23.184566),
        // optimal_route_length: 23338598,
        // optimal_route_node_count: 2395,
        description: "from Mediterrian Sea to Red Sea".to_string()
    };
    let coords_med_to_black_sea = Route {
        start: Coordinates(30.893599, 42.7491),
        end: Coordinates(18.01811, 35.002186),
        // optimal_route_length: u32::MAX,
        // optimal_route_node_count: usize::MAX,
        description: "from Mediterrian Sea to Black Sea".to_string()
    };
    let coords_indic_to_pacific = Route {
        start: Coordinates(89.605064, -7.3356276),
        end: Coordinates(137.78972, 13.273782),
        // optimal_route_length: 6625982,
        // optimal_route_node_count: 678,
        description: "from Indic to Pacific over Indonesia".to_string()
    };
    let coords_atlantic_to_indic = Route {
        start: Coordinates(-40.122555, 40.648052),
        end: Coordinates(79.83727, -11.218551),
        // optimal_route_length: 18569094,
        // optimal_route_node_count: 1909,
        description: "from Atlantic to Indic around Afrika".to_string()
    };
    let coords_east_to_west = Route {
        start: Coordinates(177.23738, 38.280342),
        end: Coordinates(-155.37436, 32.565235),
        // optimal_route_length: 37105730,
        // optimal_route_node_count: 3851,
        description: "from 177°W to 155°E".to_string()
    };
    // let coords_ = Route {              // from  to 
    //     start: Coordinates(),
    //     end: Coordinates(),
    //     optimal_route_length: ,
    //     optimal_route_node_count: 
    // };

    run_routing(graph, ch_graph, coords_med_to_red_sea);
    run_routing(graph, ch_graph, coords_med_to_black_sea);
    run_routing(graph, ch_graph, coords_indic_to_pacific);
    run_routing(graph, ch_graph, coords_atlantic_to_indic);
    run_routing(graph, ch_graph, coords_east_to_west);

}

fn run_routing(graph: &Graph, ch_graph: &Graph, route: Route) {
    let src_node = graph.closest_node(&route.start);
    let tgt_node = graph.closest_node(&route.end);

    println!("Routing {}, i.e. {:?} - {:?}", route.description, src_node, tgt_node);
    let dijkstra_result = run_dijkstra(src_node, tgt_node, graph);
    println!("{}", dijkstra_result);
    println!("Query\tTime\tVisited nodes");
    println!("DI\t{}\t{}", dijkstra_result.calculation_time, dijkstra_result.visited_nodes);
    let a_star_result = run_a_star(src_node, tgt_node, graph);
    println!("A*\t{}\t{}", a_star_result.calculation_time, a_star_result.visited_nodes);
    assert_eq_routings(&dijkstra_result, &a_star_result);
    let bidirectional_dijkstra = run_bidirectional_dijkstra(src_node, tgt_node, graph, true);
    println!("BD\t{}\t{}", bidirectional_dijkstra.calculation_time, bidirectional_dijkstra.visited_nodes);
    assert_eq_routings(&dijkstra_result, &bidirectional_dijkstra);
    let ch_result = run_ch(src_node, tgt_node, ch_graph);
    println!("CH\t{}\t{}", ch_result.calculation_time, ch_result.visited_nodes);
    assert_eq_routings(&dijkstra_result, &ch_result);
}

fn assert_eq_routings(routing_1: &ShortestPathResult, routing_2: &ShortestPathResult) {
    assert_eq!(routing_1.distance, routing_2.distance);
    match routing_1.path {
        Some(ref _path) => {
            //assert_eq!(_path.len(), routing_2.path.as_ref().unwrap().len());
        },
        None => {
            assert!(routing_2.path.is_none());
        },
    }
}
