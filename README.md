# Shiprouting on OSM data
## Programming language
Our code is written in Rust. We decided so, to get new experiences and to use a language which suited for the project. Rust aroused our interest due to its safety, speed and modernity. The strict of this language with concepts which where new for us, brought us sometimes to our limit of coding skills. But after understanding some important concepts of this language I am happy to have it chosen. I didn't measure anything to compare, but the lightweight structs and pointer handling convince me to be a fast but safe solution compared to other languages.

## General to source code
There are three source code folders. One for preprocessing (like reading PBF and generate graph). The second is for routing algorithms. And the third contains structures, which are used from both (e.g. the graph struct).

In preprocessing we read the osm file, execute a point in polygon test on it to finally generate an ocean graph for ship routing. In the execution folder (osm) we import graph, offer an leaflet frontend in an desktop application and offer a different routing algorithms on our ocean graph.

In the screenshot folder we placed some pictures of the graph and routings.

Due to the many computational heavy operations, we tryied to offer a regular logging about the happenings to keep the user updated on the progress.
The source code contains still many prints in comments to offer precise debugging if needed.

All the cli commands to run _cargo_ should be executed in the root folder of the project.

## Notes to our graph generation
Due to the size of this problem, already the generation of our graph is a bottle neck if not implemented well. So we spent time to find solutions for PBF file interpretion, for the point in polygon test and for the graph connection, which have feasable and comfortable timings.

### PBF import
The PBF file interpretation works with multithreading, which connect all objects of the file in a linked list, to speedup the many list chainings. We struggled here a bit with the opportunities of Rust, to read the coastline and its coordinates in one file read. In the end I need on my machine 133 sec for the coastline node ids, 280 sec for reading the coordinates and finally 77 sec to map the information together, which makes about 490 sec overall (for the PBF with the worlds coastlines).

### Point in polygon test
The biggest time effort for myself was spent for checking if a coordinate is on land or in water. The fist naive implementation of a odd-or-even rule algorithm took 7400ms in average. Which would mean 523 days for 6000000 tests. So some strong speedups were needed. In the following I describe a bit of my procedure. To makes it easier, I will use the term island for every polygon which is formed by coastlines. This represents islands and continents in our world. To realize a faster point in polygon test, a preprocessings for the islands is needed.

1. I started with precalculating the center of an island and the longest distance to one of its coastlines. If a coordinate is too far from its center, it can't be on the island. For big and longly shaped islands, I calculated multiple center points. Overall it brough me a speedup of almost 20000%, so a single test in 380ms.
2. Then I implemented a bounding box, like described in [[2]](#2). This dominated the first idea, by being again a bit faster with less preprocessing.
3. After that the data and code structure got more in focus. With using better suited data structures and method inlining I got again a speedup of 200%, in the end with 115 ms each test.
4. Now the test was fast if the coordinate was in no bounding box (~20ms). But being in the box of eurasia or america needs more than 200ms. So my next idea was to split all islands in many vertical parts. Every vertical part is an entry of an array. When doing the test, only coastlines in the refering vertical part will be checked. This brought a speedup of 12000% on top, so ending with a test of 9,5ms. Now the calculation was feasable within 16 hours.
5. After that, the slow checks were the ones which don't touch any island, because for every island of the world the bounding box check is needed. So I introduced a grid over the whole world with grid cells about similar size. For every grid cell out of 1654, I saved a reference to all islands which contain a part of this cell. For the test, I just check the islands which are a part of the same grid cell. I had again a speedup of 10000%, ending with 0,96 ms.
6. Now added only a few small things, which let me ending with 0,15 ms for each test in average. So overall about 15 min for 6000000, which are needed to generate 4000000 points in water.

The whole preprocessing takes about 60 sek.

It can be argumented, that the polygon test is now in O(1), with a leak of proof and being sure of the argumentation.
The world is splitted into grids. To find the right grid by coordinates is possible in O(1).
Now the question is how many islands can be in the grid cell. The number of cells is static here, but the worlds number of islands too. With argumenting, to raise the number of grid cells according to number of islands, it is maybe possible to argue, that the number of islands for each cell stays static in average. For this argumentation we can use the properties of island polygons, which are always planar and never intersecting. So the overall land surface can be never bigger than the surface of the whole planet. But we have to take into account, that the memery will raise Ω(A) where A is the surface of the planet.
In assumption, that the we can access in time O(1) a number O(1) islands, we only have to check if a test with one island is O(1). Our islands are splitted into vectors of different longitudes. If the splits depend on the number of points of the island, we can reach O(1) points in each segment. So we have to check for every island only O(1) edges.

For the last part my own implementation suits not the exactly the O(1) effort anymore, but in my opinion it is not far away.

### Graph generation
To make the graph connection with the generated points feasable in a few minutes, a grid over the whole world is used. To find the closest neighbours, only the grid cell of the node itself and the neighbour grid cells will checked. And neighbour grid cells will be only checked, if they could be reached. For this step I need about 5 min on my computer.

## My routing solutions
In this project 4 different routings are implemented. The first is the basic Dijkstra algorithm implemented with a binary heap. The other algorithms get an on section in the following.

### Birectional Dijkstra
The bidirectional Dijkstra can handle symmetric and assymetric graphs. On a symmetric graph the break condition is different to have a little speedup for such graphs, which is given four our ocean graph.

### A*
A* offers a good speedup for little changes. This routing algorithm is suited for this problem, because the graph is build on the distance metric. With the direct distance between two nodes, a good and simple heuristic can be used. It is implemented like Dijkstra on binary heap.

My implementation is a basic one. So far measuring the direct distance as heuristic takes relatively much time, due to its sinus, cosinus und arctangent computations. With a precalculation and a lookup table for those functions a better speedup can be implemented.

### Contraction Hierarchies
The Contraction Hierarchies algorithm is the most complex speedup technique in my work. It contains the contraction as a precalculation and a query algorithm.

#### Precalculation
This algorithm contracts the nodes like in typicla Contraction Hierarchy implementations. As a heuristic for the importance of nodes, which is used for the contraction order, I use following formula like in [[1]](#1):
$I(x) = L(x) + \frac{|A(x)|}{|D(x)|} + \frac{\sum_{a∈A(x)}{h(a)}} {\sum_{a∈D(x)}{h(a)}}$

$L(x)$ represents the minum level of a node. So it is contininously updated with $L(y) = max\{L(y), L(x) + 1\}$ always if x is is contracted and there exists an edge from x to y.
$D(x)$ is the set of edges, which would be removed if x got contracted and $A(x)$ is the set of edges, which would be inserted if x got contracted. $h(a)$ is the hopcount of an edge. So the number of original edges which are represented by this one.

So to calculate the importance of the nodes, a witness search between all pair of neighbeours of every single node is needed. This leeds to the most computational effort of the contraction algorithm. Further notes to this topic are mentioned later.

After calculating the importance of all nodes the speedup of _independent set_ is used. So all nodes, beginning the least importance up to the first node which is neighbour of one with lower importance, are contracted simultaniously.

After contraction, only the neighbours of the contracted nodes have to updated. And for them only their new neigbours have to be checked by witness search.

This goes on until only one node is remaining or we define another limit to stop earlier. This makes sense when calculating for the whole worlds ocean graph, because it will take too long to compute it completely.

To make the precalculation feasable, many speedups from the first working implementation were needed. I present hera a few of my ideas. For the speedups which came after my first working implementation, I can offer information about its effect on the computation time:
1. New graph structure, which enables fast inserting and removing nodes and edges. So often HashSets and HashMaps instead of Vec.
2. Binary heap for importance, to iterate fast from lowest importance upwards.
3. Calculate witness search only for new neigbours. &rarr; reduced computation time by 50%
4. Before the witness search I check if the nodes are already neighbours. Due to the optimality of every edge in our base graph, we know that they have already the best connection. &rarr; reduced computation time by 25%
5. I use A* for the witness search &rarr; reduced computation time by 25%
6. To reduce initialization effort of A*. &rarr; reduced computation time by 5%

Additionally a method is implented to restore the last precalculation session and to go on. So the long time can be spreaded over different days. It can be also stopped to go on with a more efficient way instead starting the new algorithm from zero again. The session backup is done every x minutes (where x is the time of saving operation * 50).

Still the contraction needs a long time for a graph with 4000000 nodes. Due to a bug in the base graph I had to start the calculations short hand before project finish. So far I contracted 77,5% of the graph in 13 hours calculation. With this it is not forcibly an advantage over other routing algorithms.

Further speedups will be probably needed to get close to 100%. One idea is to save results of witness searches to avoid computing distances and routings which were already made. For this implementation finally the time is missing and probably I will get close the my memory limits. Another idea would be to accelerate the A* witness search by a lookup table, like in the A* section described.

To test the contraction hierarchies algorithm completely I run it on a subgraph, containing only the black sea, which showed good results. If the graph of the black sea is present, it can be run by specifying _black sea_ after the cli command as the graph name (e.g. ´cargo run -p ship-routing test black_sea´ or ´cargo run -p ship-routing ch black_sea´).

#### Query
The query is just a reuse of the bidirectional Dijkstra on the upwarded directed graph out of the preprocessing. The only difference is, that now it should be payed attention, that this graph has arcs instead of bidirectional edges.
To get a higher query performance a bidirectional A* could be implemented.



## Run and compile
### Fast Execution
To run the final application you just have to execute the exe file in .\target\debug\ship-routing.exe. It will use the CH query.

### Compile and run source code
#### Preprocessing
We give times to estimate roughly how much time it can take to execute this step.
The time is measured on a i7-6500U CPU.
1. Place input PBF file into ./data folder.
2. Import PBF: Execute ´cargo run -p preprocessing import {filename}´ in root folder (approx. 8 min)
3. Generate Graph: Execute ´cargo run -p preprocessing generate´ (approx. 23 min)
4. Preprocessing CH: execute ´cargo run -p ship-routing ch_precalc {graphname} {nodelimit}´ (approx. 6 h for 75% of the nodes)
To continue last session of contraction hierarchie precalculations: execute ´cargo run -p ship-routing continue_ch_precalc {graphname} {nodelimit}´.
The _graphname_ (default=´graph´) will load the _graphname_.bin or _graphname_.fmi as graph. The _nodelimit_ (default=1) defines when to stop the contraction.

#### Execution
execute ´cargo run -p ship-routing di´ for Dijkstra
execute ´cargo run -p ship-routing bd´ for bidirectional Dijkstra
execute ´cargo run -p ship-routing a+´ for A*
execute ´cargo run -p ship-routing ch´ for Contraction Hierarchies query

or

Copy graph.fmi/ch_graph.bin file into .\target\debug\data\graph\
execution: run exe file in .\target\debug\ship-routing.exe (wich executes the CH query)

### Results
To test all four routing queries on a given set of challenges, you can run ´cargo run -p ship-routing test´. I got following results (Time in ms):

Routing from Mediterrian Sea to Red Sea:
| Query   |      Time      |  Visited nodes |
|----------|-------------:|------:|
DI      |14015   |3513994
A*      |4870    |1045569
BD      |7003    |8599255
CH      |9170    |21522339

Routing from Mediterrian Sea to Black Sea:
| Query   |      Time      |  Visited nodes |
|----------|-------------:|------:|
DI      |342     |15406
A*      |356     |2648
BD      |553     |76373
CH      |591     |85465

Routing from Indic to Pacific over Indonesia:
| Query   |      Time      |  Visited nodes |
|----------|-------------:|------:|
DI      |4993    |804883
A*      |965     |55884
BD      |4408    |2909902
CH      |3787    |4786017

Routing from Atlantic to Indic around Afrika:
| Query   |      Time      |  Visited nodes |
|----------|-------------:|------:|
DI      |19622   |3178215
A*      |7980    |730913
BD      |13712   |10979894
CH      |7878    |19471214

Routing from 177°W to 155°E, over the date border:
| Query   |      Time      |  Visited nodes |
|----------|-------------:|------:|
DI      |968     |220752
A*      |389     |14420
BD      |896     |613514
CH      |949     |1376855

Unfortunately CH can't give always the best results. I explain it so far with mostly following reasons:
1. The contraction went only up to 78%
2. Potential of the query to be faster, for example using bidirectional A*
Even if CH is the fastest, for example for routing 4 (from Atlantic to Indic around Afrika), it visits the most nodes. I am sure this can be optimized by a better query.

## Conclusion
For the graph generation much of effort was put to reach comfortable time for calculating new graphs.
For the routing, I implemented 3 additional queries next to basic Dijkstra. Bidirectional Dijkstra gives optimizes for the most cases a bit. Better results are offered by A*. By an easy implementation strong performance gain can be reached. Often we are 3 to 5 times faster than the basic Dijkstra.
The contraction hierarchies algorithm is implemented with preprocessing and query. The contraction takes long, but contracting ~80% feasable in half a day. The query leads to correct results, but could be faster. Reasons are the contraction, which is not complete and an unoptimized query. Ideas to go on are given.

## References
<a id="1">[1]</a> 
Julian Dibbelt, Ben Strasser and Dorothea Wagner (2015). 
Customizable Contraction Hierarchies. 
arXiv:1402.0402v5, page 5.

<a id="2">[2]</a> 
Jian Chang et al
Next Generation Computer, Animation Techniques.
https://doi.org/10.1007/978-3-319-69487-0_5, page 60