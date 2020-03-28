import * as d3 from 'd3'
import * as topojson from 'topojson';

(async function () {

    // Adapted from https://observablehq.com/@d3/zoom-to-bounding-box

    //const width = 975, height = 610;
    const width = 650, height = 400;

    // create path variable
    var path = d3.geoPath();

    let us = await d3.json("data/states-albers-10m.json");
    //console.log(us);
    var svg = d3.select("#map").append('svg')
        .attr('width', width)
        .attr('height', height);

    var g = svg.append("g");
    g.attr('transform', 'scale(0.6)');

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
})();