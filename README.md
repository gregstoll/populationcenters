# populationcenters
Find where to put things close to most of the US's population

Let's say you're an employee at a major theme park company, and you're about to launch a new kind of theme park that everyone in the US is going to want to visit.  Where can you build it that minimizes the root mean squared distance for every person in the US?  What if you build 2 or 3 copies of the theme park?

See the results at TODO

# How to get the data
## County centroids
First I needed to get the geographic center of each county in the US.  From [census.gov's 
TIGER/Line Shapefiles page](https://www.census.gov/cgi-bin/geo/shapefiles/index.php) I downloaded a shapefile (.shp) for all the counties (and equivalents) in the US.  Then I ran
```
npx -p shapefile shp2json tl_2019_us_county.shp > county_shapes.json
```
to use `shp2json` from the [shapefile package](https://github.com/mbostock/shapefile) to convert the shapefile to GeoJSON.

## County populations
I downloaded the county population data from [census.gov](https://data.census.gov/cedsci/table?q=population%20by%20county&g=0100000US.050000&tid=ACSDP5Y2018.DP05&hidePreview=true) (note that this page is pretty memory/CPU-intensive on my browser; it may be better to go to [data.census.gov and search for "population by county"](https://data.census.gov/cedsci/all?q=population%20by%20county&hidePreview=false&tid=ACSDP1Y2018.DP05). I took the latest population column for each county and saved that to [census_county_data.tsv](https://github.com/gregstoll/populationcenters/blob/master/census_county_data.tsv).

## Merging them
[calculate_centroids_and_merge_population.js](https://github.com/gregstoll/populationcenters/blob/master/calculate_centroids_and_merge_population.js) reads in these two input files and joins them together (as they both have GeoID's for counties) into [data/county_centroids.json](https://github.com/gregstoll/populationcenters/blob/master/data/county_centroids.json).  This is the only input that [find_nearest_counties.rs](https://github.com/gregstoll/populationcenters/blob/master/find_nearest_counties.rs) needs.

# Implementation
This was a fun exercise in optimization.

Unless otherwise noted, all times were taken on my laptop with a 4-core Intel core i7-8650U 1.9Ghz.  I just ran each of these once or twice, so don't take the times too seriously!

## Straightforward implementation
I started with a straightforward implementation where we brute-force calculate the distance every time.

Code is at revision [7ad3c5a3](https://github.com/gregstoll/populationcenters/blob/7ad3c5a37e43508f324cd03a6e77760dfef2af9c/find_nearest_counties.rs).

- 1 county: 0.7 seconds
- 2 counties: 1164.5 seconds ~= 19.5 minutes (this seems about right, should be around (3000/2)\*2 times longer)
- 3 counties: didn't even try it, seems like it would have taken around (3000/3)\*(3/2) longer, which is around 40 days!

## Parallel implementation
This is an [embarrassingly parallel](https://en.wikipedia.org/wiki/Embarrassingly_parallel) problem, and I used [Rayon](https://github.com/rayon-rs/rayon) to parallelize it.

Code is at revision [37c03437](https://github.com/gregstoll/populationcenters/blob/37c03437ca92114702dab6e40c2376fdcf102f9c/find_nearest_counties.rs).

- 1 county: 0.1 seconds
- 2 counties: 343.5 seconds (~3.4x speedup over the non-parallel version for an 4 core machine)
- 3 counties: this would have used a ton of memory, because the way it was implemented would require a giant Vec for all the possible combinations.  More on this later.

Memoize squared distance between counties in a HashMap
1 county: 4.06 seconds (uh-oh)
2 counties: ...

had to put in a Mutex, get rid of that by not making it static

next try using a big Array?
