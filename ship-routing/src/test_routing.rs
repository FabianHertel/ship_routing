use graph_lib::{Graph, Coordinates};

use crate::{dijkstra::run_dijkstra, a_star::run_a_star, bidirectional_dijkstra::run_bidirectional_dijkstra};

struct Route {
    start: Coordinates,
    end: Coordinates,
    optimal_route_length: f32,
    optimal_route_node_count: usize,
    description: String
}

// DEPEND ON THE GRAPH
pub fn test_samples(graph: &Graph) {
    let coords_med_to_black_sea = Route {
        start: Coordinates(30.893599, 42.7491),
        end: Coordinates(18.01811, 35.002186),
        optimal_route_length: f32::MAX,
        optimal_route_node_count: usize::MAX,
        description: "from Mediterrian Sea to Black Sea".to_string()
    };
    let coords_med_to_red_sea = Route {
        start: Coordinates(28.677027, 33.622692),
        end: Coordinates(37.114494, 23.184566),
        optimal_route_length: 23338.598,
        optimal_route_node_count: 2395,
        description: "from Mediterrian Sea to Red Sea".to_string()
    };
    let coords_indic_to_pacific = Route {
        start: Coordinates(89.605064, -7.3356276),
        end: Coordinates(137.78972, 13.273782),
        optimal_route_length: 6625.982,
        optimal_route_node_count: 678,
        description: "from Indic to Pacific over Indonesia".to_string()
    };
    let coords_atlantic_to_indic = Route {
        start: Coordinates(-40.122555, 40.648052),
        end: Coordinates(79.83727, -11.218551),
        optimal_route_length: 18569.094,
        optimal_route_node_count: 1909,
        description: "from Atlantic to Indic around Afrika".to_string()
    };
    let coords_east_to_west = Route {
        start: Coordinates(177.23738, 38.280342),
        end: Coordinates(-155.37436, 32.565235),
        optimal_route_length: 37105.73,
        optimal_route_node_count: 3851,
        description: "from 177°W to 155°E".to_string()
    };
    // let coords_ = Route {              // from  to 
    //     start: Coordinates(),
    //     end: Coordinates(),
    //     optimal_route_length: ,
    //     optimal_route_node_count: 
    // };

    run_routing(graph, coords_med_to_red_sea);
    run_routing(graph, coords_med_to_black_sea);
    run_routing(graph, coords_indic_to_pacific);
    run_routing(graph, coords_atlantic_to_indic);
    run_routing(graph, coords_east_to_west);

}

fn run_routing(graph: &Graph, route: Route) {
    let src_node = graph.closest_node(&route.start);
    let tgt_node = graph.closest_node(&route.end);

    println!("Routing {}, i.e. {:?} - {:?}", route.description, src_node, tgt_node);
    let dijkstra_result = run_dijkstra(src_node, tgt_node, graph);
    println!("DI: {}", dijkstra_result);
    let a_star_result = run_a_star(src_node, tgt_node, graph);
    println!("A*: {}", a_star_result);
    let bidirectional_dijkstra = run_bidirectional_dijkstra(src_node, tgt_node, graph);
    println!("BD: {}", bidirectional_dijkstra);

    assert!((route.optimal_route_length - dijkstra_result.distance).abs() < 0.001);
    match dijkstra_result.path {
        Some(path) => assert_eq!(route.optimal_route_node_count, path.len()),
        None => assert_eq!(route.optimal_route_node_count, usize::MAX)
    }
}
