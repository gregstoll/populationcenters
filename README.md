# populationcenters
Find where to put things close to most of the US's population

TODO better description

This was a fun exercise in optimization.

(note that all times were taken on my laptop, don't take them too seriously)

## Straightforward implementation
I started with a straightforward implementation where we brute-force calculate the 
distance every time.

1 county: 0.7 seconds
2 counties: 1164.5 seconds (this seems about right, should be (3000/2)*2 slower?)
3 counties: didn't even try it

Parallelize it!
1 county: 0.1 seconds
2 counties: 343.5 seconds (~3.4x speedup for an 8 core machine)
3 counties: way it's implemented would require a giant Vec

Memoize squared distance between counties in a HashMap
1 county: 4.06 seconds (uh-oh)
2 counties: ...

had to put in a Mutex, get rid of that by not making it static

next try using a big Array?