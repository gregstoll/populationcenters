var fs = require('fs');
var d3Geo = require('d3-geo');


fs.readFile("county_shapes.json", 'utf8', function (err, contents) {
    if (err) {
        return console.error(err);
    }
    console.info("Length: " + contents.length);
    const json = JSON.parse(contents);
    const features = json.features;
    console.info("Features: " + features.length);
    let stateCounts = {};
    let centroidData = [];
    for (let county of features)
    {
        const properties = county.properties;
        console.info("geoid: " + properties.GEOID + ", name: " + properties.NAME + ", statefp: " + properties.STATEFP);
        if (!stateCounts.hasOwnProperty(properties.STATEFP)) {
            stateCounts[properties.STATEFP] = 0;
        }
        stateCounts[properties.STATEFP] += 1;
        const geometry = county.geometry;
        const centroid = d3Geo.geoCentroid(geometry).toString();
        //console.info(centroid);
        centroidData.push({"geoid": properties.GEOID, "state": properties.STATEFP, "centroid": centroid});
    }
    fs.writeFile("county_centroids.json", JSON.stringify(centroidData), 'utf8', function (err) {
        if (err) {
            return console.error(err);
        }
    });

    for (let stateFP in stateCounts)
    {
        console.info("State: " + stateFP + " Counties: " + stateCounts[stateFP]);
    }
    console.info("number of states: " + Object.keys(stateCounts).length);
});