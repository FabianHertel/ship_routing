use graph_lib::{Graph, Coordinates};

use crate::dijkstra::run_dijkstra;

struct Route {
    start: Coordinates,
    end: Coordinates,
    optimal_route_length: u32,
    optimal_route_node_count: usize
}

// DEPEND ON THE GRAPH
pub fn test_samples(graph: &Graph) {
    let coords_med_to_black_sea = Route {              // from Mediterrian Sea to Black Sea
        start: Coordinates(30.893599, 42.7491),
        end: Coordinates(18.01811, 35.002186),
        optimal_route_length: u32::MAX,
        optimal_route_node_count: usize::MAX
    };
    let coords_med_to_red_sea = Route {              // from Mediterrian Sea to Red Sea
        start: Coordinates(28.677027, 33.622692),
        end: Coordinates(37.114494, 23.184566),
        optimal_route_length: 23338598,
        optimal_route_node_count: 2395
    };
    let coords_indic_to_pacific = Route {              // from Indic to Pacific over Indonesia
        start: Coordinates(89.605064, -7.3356276),
        end: Coordinates(137.78972, 13.273782),
        optimal_route_length: 6625982,
        optimal_route_node_count: 678
    };
    let coords_atlantic_to_indic = Route {              // from Atlantic to Indic around Afrika
        start: Coordinates(-40.122555, 40.648052),
        end: Coordinates(79.83727, -11.218551),
        optimal_route_length: 18569094,
        optimal_route_node_count: 1909
    };
    let coords_east_to_west = Route {              // from 177°W to 155°E
        start: Coordinates(177.23738, 38.280342),
        end: Coordinates(-155.37436, 32.565235),
        optimal_route_length: 37105730,
        optimal_route_node_count: 3851
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
    let dijkstra_result = run_dijkstra(src_node, tgt_node, graph);
    println!("Finished dijkstra from {} to {} with {}", src_node.id, tgt_node.id, dijkstra_result);
    assert!((route.optimal_route_length as i32 - dijkstra_result.distance as i32) < 10000);
    match dijkstra_result.path {
        Some(path) => assert!((route.optimal_route_node_count as i32 - path.len() as i32) < 100),
        None => assert_eq!(route.optimal_route_node_count, usize::MAX)
    }
}
