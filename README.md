# General
There are two src code folders. One for preprocessing and one for the final execution.
In preprocessing we read the osm file, execute a point in polygon test on it to finally generate an ocean graph for ship routing.
In the execution folder (osm) we import graph, offer an leaflet frontend in an desktop application and offer a dijkstra algorithm on oceans.

The jpg in the root folder shows a small horizontal part of the graph. We were not able to display more.

# Run and compile
To run the final application you just have to execute the exe file in .\execution\src-tauri\target\debug\osm.exe

## Compile and run source code
### Preprocessing
Place input PBF file into ./data folder.
Import PBF: Execute ´cargo run -p preprocessing import {filename}´ in root folder
Generate Graph: Execute ´cargo run -p preprocessing generate´ in root folder
Preprocessing CH: execute ´cargo run -p ship-routing ch_precalc´ in root folder (CAN TAKE MANY HOURS /DAYS!)
To continue last session of contraction hierarchie precalculations: execute ´cargo run -p ship-routing continue_ch_precalc´ in root folder

### Execution
execute ´cargo run -p ship-routing´

or

Copy graph.fmi/graph.bin file into .\target\debug\data\graph\
execution: run exe file in .\target\debug\ship-routing.exe