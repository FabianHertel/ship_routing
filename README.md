# General
There are two src code folders. One for preprocessing and one for the final execution.
In preprocessing we read the osm file, execute a point in polygon test on it to finally generate an ocean graph for ship routing.
In the execution folder we import graph, offer an leaflet frontend in an desktop application and offer a dijkstra algorithm on oceans.

# Run and compile
To run the final application you just have to execute the exe file in .\execution\src-tauri\target\debug\osm.exe

## Compile and run source code
### Preprocessing
navigate into the preprocessing folder
execute ´cargo install´ to install the packages
execute ´cargo run import {filename}´ to import a osm (f.e. ´cargo run import ../../planet-coastlinespbf-cleanedosm.pbf´ when placed next to project)
execute ´cargo run generate´ to generate a graph

copy the graph.txt to the execution folder into 'execution/src-tauri/src/'

### Execution
navigate into the execution folder
execute ´cargo install tauri-cli´
execute ´cargo tauri build´
to run it in dev mode: ´cargo tauri dev´
exectution: run exe file in .\execution\src-tauri\target\debug\osm.exe