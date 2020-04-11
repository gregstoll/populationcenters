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

# Implementation and optimization
We assume that the entire population of a county is located at its center, so to do the calculation we simply try every combination of _n_ counties and for each one, iterate over every county in the country and calculate the distance squared (to the closest member of the combination) multiplied by the population of the county.

(I'm not sure if we should be squaring the population of the county, too?)

This was a fun exercise in optimization!

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

## Memoize squared distance between counties in a HashMap
The most expensive part of the calculation is calculating the distance between two coordinates.  (it involves a handful of trigonometric functions and square roots, see [find_distance_between_coordinates()](https://github.com/gregstoll/populationcenters/blob/master/find_nearest_counties.rs#L188) )  Since there are only 3100 counties, there are only 9.6 million pairwise distances, and if we store an f64 for each, that takes 8 bytes * 9.6 million = ~76 MB of memory, which is easily doable.  (really you can only store half that much since distance is symmetric, but I didn't bother)

So we can use a HashMap to map each pair of coordinates to the squared distance between them.  In this initial implementation I made a number of suboptimal choices:
- I made the HashMap a static variable (using the [lazy_static](https://lib.rs/crates/lazy_static) crate) - this meant I had to wrap the HashMap in a Mutex, which meant acquiring that Mutex every time we access the HashMap.
- The type of the HashMap's key was a tuple of Coordinates, which meant I had to write a hash function for f64's.  I don't think caused any problems, but it's weird and not a good sign...
- I don't remember why now, but I couldn't run this in parallel.

Code is at revision [ad7aa410](https://github.com/gregstoll/populationcenters/blob/ad7aa4103ea2419bba959ca08f997e4b70bc4c6d/find_nearest_counties.rs)

- 1 county: 4.1 seconds (uh-oh, not a good sign)
- 2 counties: over an hour before I gave up

## Memoize squared distance between counties in a non-static HashMap
I addressed the static part by initializing the HashMap and passing down a non-mutable reference when we do the calculations.  This meant it didn't need to be wrapped in a Mutex anymore.

Unfortunately I didn't commit this code.

- 1 county: 7.8 seconds (yikes!)
- 2 counties: over an hour before I gave up

## Memoize squared distance between counties in a Vec<> (non-parallel)
The HashMap is actually more powerful than we need here - if we just use the index of each county as a "key", we can keep all the distances in a Vec<> and look them up much more easily.

Code is at revision [f8a0cd36](https://github.com/gregstoll/populationcenters/blob/f8a0cd36ac03d077c57ccad54bf910351342fcd5/find_nearest_counties.rs)

- 1 county: 0.7 seconds
- 2 counties: 372 seconds (6.2 minutes) - this is ~3x faster than the previous fastest non-parallel version!

## Memoize squared distance between counties in a Vec<> (parallel)
Per the note above (under "Parallel implementation"), I wasn't able to run 

I also ran this on my desktop machine (an Intel 6 core i5-8600K at 3.6Ghz)
- 1 county: 
- 2 counties: 

This was cool because I could see all my CPUs get pegged at 100% :-)  It was also neat to see the desktop machine be significantly faster, because in the non-parallel case it's a little slower despite having a higher clock speed.  (I guess i7 versus i5 makes a difference!)

TODO - desktop timing?

TODO - chunk Vec's

TODO - make DistanceCache weighted by population
