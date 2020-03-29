import * as d3 from 'd3'
import * as topojson from 'topojson';

(async function () {

    // Adapted from https://observablehq.com/@d3/zoom-to-bounding-box

    const width = 975, height = 610;

    // create path variable
    // Scale is per https://github.com/topojson/us-atlas
    let projection = d3.geoAlbers().scale(1300).translate([width/2, height/2]);
    let path = d3.geoPath();

    let us = await d3.json("data/states-albers-10m.json");
    let mapDiv = d3.select('#map');
    mapDiv.attr('style', 'transform: scale(0.6)');
    let svg = mapDiv.append('svg')
        .attr('width', width)
        .attr('height', height);

    let g = svg.append("g");
    let pointsG = svg.append("g");

    let states = us.objects.states;
    states.geometries = states.geometries.filter(state => state.properties.name !== "Alaska" && state.properties.name !== "Hawaii");

    // Draw states
    g.selectAll("path")
        .data(topojson.feature(us, states).features)
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

    let countyData = await d3.json("data/county_centroids.json");
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
    // https://stackoverflow.com/questions/20987535/plotting-points-on-a-map-with-d3
    g.selectAll(".pin")
        .data(countyData)
        .enter()
        .append("circle", ".pin")
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
})();