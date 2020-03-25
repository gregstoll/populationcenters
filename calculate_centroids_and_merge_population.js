var fs = require('fs');
var d3Geo = require('d3-geo');
var d3Dsv = require('d3-dsv');


fs.readFile("county_shapes.json", 'utf8', function (err, contents) {
    if (err) {
        return console.error(err);
    }
    fs.readFile("census_county_data.tsv", 'utf8', function(err, censusContents) {
        if (err) {
            return console.error(err);
        }
        const censusRows = d3Dsv.tsvParse(censusContents);
        let countyPopulations = new Map();
        for (let censusRow of censusRows)
        {
            const countyId = censusRow["id"].substr(censusRow["id"].length - 5);
            countyPopulations.set(countyId, parseInt(censusRow["Total population"], 10));
        }
        console.info("Length: " + contents.length);
        const json = JSON.parse(contents);
        const features = json.features;
        console.info("Features: " + features.length);
        let stateCounts = {};
        let centroidData = [];
        let noPopulationCount = 0;
        for (let county of features)
        {
            const properties = county.properties;
            console.info("geoid: " + properties.GEOID + ", name: " + properties.NAME + ", statefp: " + properties.STATEFP);
            if (!stateCounts.hasOwnProperty(properties.STATEFP)) {
                stateCounts[properties.STATEFP] = 0;
            }
            stateCounts[properties.STATEFP] += 1;
            let population = 0;
            if (countyPopulations.has(properties.GEOID))
            {
                population = countyPopulations.get(properties.GEOID);
            }
            else
            {
                noPopulationCount++;
            }
            console.info("population: " + population);
            const geometry = county.geometry;
            const centroid = d3Geo.geoCentroid(geometry).toString();
            //console.info(centroid);
            centroidData.push({"geoid": properties.GEOID, "state": properties.STATEFP, "centroid": centroid, "population" : population});
        }
        fs.writeFile("county_centroids.json", JSON.stringify(centroidData, undefined, '\t'), 'utf8', function (err) {
            if (err) {
                return console.error(err);
            }
        });

        for (let stateFP in stateCounts)
        {
            console.info("State: " + stateFP + " Counties: " + stateCounts[stateFP]);
        }
        console.info("number of states: " + Object.keys(stateCounts).length);
        console.info("counties with no population: " + noPopulationCount);
    });
});