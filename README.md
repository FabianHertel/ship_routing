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
I implemented 3 additional queries next to basic Dijkstra. Bidirectional Dijkstra gives optimizes for the most cases a bit. Better results are offered by A*. By an easy implementation strong performance gain can be reached. Often we are 3 to 5 times faster than the basic Dijkstra.
The contraction hierarchies algorithm is implemented with preprocessing and query. The contraction takes long, but contracting ~80% feasable in half a day. The query leads to correct results, but could be faster. Reasons are the contraction, which is not complete and an unoptimized query. Ideas to go on are given.

## References
<a id="1">[1]</a> 
Julian Dibbelt, Ben Strasser and Dorothea Wagner (2015). 
Customizable Contraction Hierarchies. 
arXiv:1402.0402v5, page 5.