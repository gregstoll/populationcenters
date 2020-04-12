import * as d3 from 'd3'
import * as topojson from 'topojson';

interface CountyCentroid {
    geoid: string,
    state: string,
    centroid: string,
    population: number
};

interface HasFeatures {
    features: any[]
}

(async function () {

    // Adapted from https://observablehq.com/@d3/zoom-to-bounding-box

    const width = 975, height = 610;

    // create path variable
    // Scale is per https://github.com/topojson/us-atlas
    let projection = d3.geoAlbers().scale(1300).translate([width/2, height/2]);
    let path = d3.geoPath();

    let us = await d3.json("data/states-albers-10m.json");
    let states = us.objects.states;
    states.geometries = states.geometries.filter(state => state.properties.name !== "Alaska" && state.properties.name !== "Hawaii");
    // 'features' isn't a declared property?
    // so tell TypeScript it's OK
    const stateFeature : any = topojson.feature(us, states);

    let countyData : CountyCentroid[] = await d3.json("data/county_centroids.json");
    //filter out Alaska/Hawaii/territories
    // 02 Alaska
    // 15 Hawaii
    // 72 Puerto Rico
    // 56 Wyoming is the last "real" state
    countyData = countyData.filter(county => {
        const state = parseInt(county.state, 10);
        if (state === 2 || state === 15) {
            return false;
        }
        return state <= 56;
    });

    drawMap('#map');
    const oneLocation = [["-99.89793552425651", "38.08749756724239"]];
    const twoLocations = [["-85.45515018740984", "35.926337261802644"],
                          ["-116.47005800749761", "38.03590529863756"]];
    const threeLocations = [["-80.76115754448554", "41.317087095771384"],
                            ["-116.47005800749761", "38.03590529863756"],
                            ["-89.30411404608736", "29.90486022173568"]];
    drawMap('#map1', getCountyCentroidsFromCoordList(oneLocation));
    drawMap('#map2', getCountyCentroidsFromCoordList(twoLocations));
    drawMap('#map3', getCountyCentroidsFromCoordList(threeLocations));
    const oneLocationNoSquare = [["-101.31204687704125", "37.19223236042145"]];
    const twoLocationsNoSquare = [["-116.17665105622207", "34.84170884534611"],
                                  ["-85.05876112828659", "36.99099027516325"]];
    const threeLocationsNoSquare = [["-80.33472080507312", "40.99117132468833"],
                                    ["-116.17665105622207", "34.84170884534611"],
                                    ["-90.72780147927472", "30.440047525709172"]];
    drawMap('#map1NoSquare', getCountyCentroidsFromCoordList(oneLocationNoSquare));
    drawMap('#map2NoSquare', getCountyCentroidsFromCoordList(twoLocationsNoSquare));
    drawMap('#map3NoSquare', getCountyCentroidsFromCoordList(threeLocationsNoSquare));

    function getCountyCentroidsFromCoordList(longLats: Array<Array<string>>) : CountyCentroid[] {
        return longLats.map(latLong => getCountyCentroidsFromCoords(latLong[0], latLong[1]));
    }
    function getCountyCentroidsFromCoords(long: string, lat: string) : CountyCentroid {
        // Relies on the fact that the lat/long are exactly the same
        let targetLongLat = long + "," + lat;
        return countyData.filter(county => county.centroid === targetLongLat)[0];
    }

    function drawMap(mapDivId: string, destinations?: CountyCentroid[]) {
        console.log("Drawing " + mapDivId + " with geoids " + destinations?.map(c => c.geoid).join(", "));
        let mapDiv = d3.select(mapDivId);
        let innerMapDiv = mapDiv.append('div');
        innerMapDiv.attr('style', 'transform: scale(0.6); transform-origin: top;');
        let svg = innerMapDiv.append('svg')
            .attr('width', width)
            .attr('height', height);

        let g = svg.append("g");

        // Draw states
        g.selectAll("path")
            .data(stateFeature.features)
            .join("path")
            .style("fill", "#ddd")
            .attr("d", path);

        // Draw state borders
        g.append("path")
            .attr("fill", "none")
            .attr("stroke", "black")
            .attr("stroke-linejoin", "round")
            // This way doesn't draw external borders
            //.attr("d", path(topojson.mesh(us, us.objects.states, (a, b) => a !== b)));
            .attr("d", path(topojson.mesh(us, states)));

        // https://stackoverflow.com/questions/20987535/plotting-points-on-a-map-with-d3
        g.selectAll(".pin")
            .data(countyData)
            .enter()
            .append("circle")
            .attr("class", "countyCircle")
            .attr("r", function(d) {
                // radius should be proportional to sqrt(population)
                // so area is proportional to population
                return Math.sqrt(d.population) / 75;
            })
            .attr("transform", function(d) {
                const parts = d.centroid.split(",");
                const projectedPoint = projection([
                    parseFloat(parts[0]),
                    parseFloat(parts[1])
                ]);
                return "translate(" + 
                    projectedPoint[0] + "," +
                    projectedPoint[1] + ")";
            });

        if (destinations !== undefined) {
            g.selectAll(".pin")
                .data(destinations)
                .enter()
                .append("circle")
                .attr("class", "countyCircle destinationCircle")
                .attr("r", 10)
                .attr("transform", function(d) {
                    const parts = d.centroid.split(",");
                    const projectedPoint = projection([
                        parseFloat(parts[0]),
                        parseFloat(parts[1])
                    ]);
                    return "translate(" + 
                        projectedPoint[0] + "," +
                        projectedPoint[1] + ")";
                });
        }

    }
})();