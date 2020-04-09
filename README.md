# populationcenters
Find where to put things close to most of the US's population

TODO better description

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

(note that all times were taken on my laptop, don't take them too seriously)

## Straightforward implementation
I started with a straightforward implementation where we brute-force calculate the 
distance every time.

- 1 county: 0.7 seconds
- 2 counties: 1164.5 seconds (this seems about right, should be (3000/2)\*2 slower?)
- 3 counties: didn't even try it

## Parallel implementation
This is an [embarrassingly parallel](https://en.wikipedia.org/wiki/Embarrassingly_parallel) problem, and I used [Rayon](https://github.com/rayon-rs/rayon) to parallelize it.

- 1 county: 0.1 seconds
- 2 counties: 343.5 seconds (~3.4x speedup for an 4 core machine)
- 3 counties: this would have used a ton of memory, because the way it was implemented would require a giant Vec for all the possible combinations.  More on this later.

Memoize squared distance between counties in a HashMap
1 county: 4.06 seconds (uh-oh)
2 counties: ...

had to put in a Mutex, get rid of that by not making it static

next try using a big Array?
