use std::convert::From;
use std::env;
use std::fs;
use std::fmt;
use std::time;
use itertools::Itertools;
use json;
use rayon::prelude::*;

static COMPUTE_IN_PARALLEL : bool = true;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = get_mode(args);
    let start_time = time::Instant::now();
    let county_datas = read_county_data();
    println!("Got {} counties", county_datas.len());
    match mode {
        Mode::FindClosest => {
            println!("{:?}", find_closest_location_to_all_counties(&county_datas, 1));
            println!("{:?}", find_closest_location_to_all_counties(&county_datas, 2));
            //println!("{:?}", find_closest_location_to_all_counties(&county_datas, 3));
        }
        Mode::CountClosestPopulation => {
            println!("Counting population");
            println!("2 locations: {:?}", count_closest_population(&county_datas, &["47185", "32023"]));
            println!("3 locations: {:?}", count_closest_population(&county_datas, &["39155", "32023", "22087"]));
            println!("2 non-squared locations: {:?}", count_closest_population(&county_datas, &["06071", "21207"]));
            println!("3 non-squared locations: {:?}", count_closest_population(&county_datas, &["42073", "06071", "22063"]));
        }
    }
    println!("took {} secs", time::Instant::now().duration_since(start_time).as_secs_f32());
}

enum Mode {
    FindClosest,
    CountClosestPopulation
}

fn get_mode(args: Vec<String>) -> Mode {
    if args.len() == 2 && args[1].to_ascii_lowercase() == "countclosestpopulation" {
        return Mode::CountClosestPopulation;
    }
    return Mode::FindClosest;
}
fn count_closest_population(counties: &[CountyData], geoids: &[&str]) -> Vec<u32> {
    let mut closest_count = Vec::new();
    let mut coords = Vec::new();
    for geoid in geoids.iter() {
        closest_count.push(0);
        coords.push(single_item(&mut counties.iter().filter(|county| county.geoid == *geoid)).coordinate);
    }
    for county in counties.iter() {
        let min_index_and_distance = coords
            .iter()
            .enumerate()
            .map(|location| (
                location.0,
                find_weighted_squared_distance_between_coordinates(&location.1, &county.coordinate, None, None, Some(1), None)))
            .fold((coords.len(), 1./0. /*Inf*/), |x, y| { if x.1 < y.1 { x } else { y }});
        closest_count[min_index_and_distance.0] += county.population;
    }
    return closest_count;
}

fn single_item<I, T>(it: &mut I) -> T
where 
    I: Iterator<Item=T>
{
    if let Some(item) = it.next() {
        if let None = it.next() {
            return item;
        }
        panic!("Too many items in iterator!");
    }
    panic!("No items in iterator!");
}

#[derive(Debug, Clone)]
pub struct CountyData {
    coordinate: Coordinate,
    index: usize,
    geoid: String,
    state: u8,
    population: u32
}

impl fmt::Display for CountyData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, index: {}, geoid: {}, state: {}, population: {}", self.coordinate, self.index, self.geoid, self.state, self.population)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Coordinate {
    longitude: f64,
    latitude: f64
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.longitude, self.latitude)
    }
}

struct DistanceCache {
    entries: Vec<f64>,
    number_of_columns: usize
}
impl DistanceCache {
    fn new(coords: Vec<(Coordinate, u32)>) -> DistanceCache {
        let mut entries: Vec<f64> = Vec::with_capacity(coords.len() * coords.len());
        for (_, (coord1, _)) in coords.iter().enumerate() {
            for (_, (coord2, pop2)) in coords.iter().enumerate() {
                let weighted_squared_distance = find_weighted_squared_distance_between_coordinates(&coord1, &coord2, None, None, Some(*pop2), None);
                entries.push(weighted_squared_distance);
            }
        }
        return DistanceCache { entries, number_of_columns: coords.len() };
    }
}


fn read_county_data() -> Vec::<CountyData> {
    let contents = fs::read_to_string("public/data/county_centroids.json").expect("Failed to open county_centroids");
    let county_parsed_json = json::parse(&contents).expect("Failed to parse JSON");
    let mut county_datas : Vec::<CountyData> =
        county_parsed_json.members().map(|value| parse_county_data(value)).filter(should_process_county).collect();
    update_county_indices(&mut county_datas);
    return county_datas;
}

fn update_county_indices(county_datas: &mut [CountyData]) {
    for (i, county) in county_datas.iter_mut().enumerate() {
        county.index = i;
    }
}

fn should_process_county(county_data: &CountyData) -> bool {
    //filter out Alaska/Hawaii/territories
    // 02 Alaska
    // 15 Hawaii
    // 72 Puerto Rico
    // 56 Wyoming is the last "real" state
    let state = county_data.state;
    return state != 2 && state != 15 && state <= 56;
}

// Assumes that update_county_indices has been called on counties
fn find_closest_location_to_all_counties(counties: &[CountyData], number_of_locations: u8) -> Vec<Coordinate> {
    let empty_vec : Vec<(usize, Coordinate)> = vec!();
    for (i, county) in counties.iter().enumerate() {
        if county.index != i {
            panic!("county at position {} has wrong index ({}) !", i, county.index);
        }
    }
    let distance_data = DistanceCache::new(counties.iter().map(|county| (county.coordinate, county.population)).collect());

    //TODO - unify these somewhat?
    if COMPUTE_IN_PARALLEL {
        let mut location_choices = counties
            .iter()
            .map(|county| (county.index, county.coordinate)).combinations(usize::from(number_of_locations));
        // To use par_iter() we need a Vec<>.  But creating a Vec<> of all the possibilities would use
        // too much memory for number_of_locations >= 3.  So process the iterator 100000 at a time.
        // each entry in iterator takes 8 (index) + (8 + 8) (Coordinate) = 24 bytes * number_of_locations, so 100000 at a time means the
        // Vec will use 2.4 GB * number_of_locations
        let mut best_location_choice = (1./0. /*Inf*/, empty_vec.clone());
        loop {
            let current_chunk : Vec<Vec<(usize, Coordinate)>> = location_choices.by_ref().take(100000).collect();
            if current_chunk.len() == 0 {
                break;
            }
            let result = current_chunk
                .par_iter()
                .map(|location_choice| (find_squared_distance_to_all_counties(&location_choice, &counties, Some(&distance_data)), location_choice))
                .reduce(|| (1./0. /*Inf*/, &empty_vec), |x, y| { if x.0 < y.0 { x } else { y }});
            if result.0 < best_location_choice.0 {
                //TODO this is kinda gross
                best_location_choice.0 = result.0;
                best_location_choice.1 = result.1.clone();
            }
        }
        return best_location_choice.1.iter().map(|(_index, location)| location.clone()).collect();
    }
    else {
        let location_choices = counties
            .iter()
            .map(|county| (county.index, county.coordinate)).combinations(usize::from(number_of_locations));

        let result = location_choices
            .map(|location_choice| (find_squared_distance_to_all_counties(&location_choice, &counties, Some(&distance_data)), location_choice))
            .fold((1./0. /*Inf*/, empty_vec), |x, y| { if x.0 < y.0 { x } else { y }});
        return result.1.iter().map(|(_index, location)| location.clone()).collect();
    }
}

fn find_squared_distance_to_all_counties<'a>(locations: &Vec<(usize, Coordinate)>, counties: &'a [CountyData], distance_data_option: Option<&DistanceCache>) -> f64 {
    // TODO - could filter out some of these and early return Inf
    let total = counties.iter().map(|county| find_squared_distance_to_single_county(locations, &county, distance_data_option)).sum();
    return total;
}

fn find_squared_distance_to_single_county<'a>(locations: &Vec<(usize, Coordinate)>, county: &'a CountyData, distance_data_option: Option<&DistanceCache>) -> f64 {
    let county_coordinate = &county.coordinate;
    let min_distance = locations
        .iter()
        .map(|location| find_weighted_squared_distance_between_coordinates(&location.1, &county_coordinate, Some(location.0), Some(county.index), None, distance_data_option))
        .fold(1./0. /*Inf*/, f64::min);
    return min_distance * min_distance;
}

fn find_weighted_squared_distance_between_coordinates(
    coord1: &Coordinate,
    coord2: &Coordinate,
    index1_option: Option<usize>,
    index2_option: Option<usize>,
    weight: Option<u32>,
    distance_data_option: Option<&DistanceCache>) -> f64 {
    if let Some(distance_data) = distance_data_option {
        if let Some(index1) = index1_option {
            if let Some(index2) = index2_option {
                return distance_data.entries[index1 * distance_data.number_of_columns + index2];
            }
        }
    }

    let distance = find_distance_between_coordinates(coord1, coord2);
    let squared_distance = distance * distance;
    return squared_distance * weight.expect("Weight needs to be specified if no distance_data_cache!") as f64;
}

/// Find the distance in km between two coordinates
fn find_distance_between_coordinates(coord1: &Coordinate, coord2: &Coordinate) -> f64 {
    // Haversine formula
    // https://rust-lang-nursery.github.io/rust-cookbook/science/mathematics/trigonometry.html#distance-between-two-points-on-the-earth
    let earth_radius_kilometer = 6371.0_f64;

    let coord1_latitude_radians = coord1.latitude.to_radians();
    let coord2_latitude_radians = coord2.latitude.to_radians();

    let delta_latitude = (coord1.latitude - coord2.latitude).to_radians();
    let delta_longitude = (coord1.longitude - coord2.longitude).to_radians();

    let central_angle_inner = (delta_latitude / 2.0).sin().powi(2)
        + coord1_latitude_radians.cos() * coord2_latitude_radians.cos() * (delta_longitude / 2.0).sin().powi(2);
    let central_angle = 2.0 * central_angle_inner.sqrt().asin();

    let distance = earth_radius_kilometer * central_angle;
    return distance;
}

fn parse_county_data(j: &json::JsonValue) -> CountyData {
    if let json::JsonValue::Object(obj) = j {
        let centroid_str = obj.get("centroid").expect("No centroid").as_str().expect("Centroid is not a string?");
        let centroid_str_parts: Vec<&str> = centroid_str.split(',').collect();
        let longitude: f64 = centroid_str_parts[0].parse::<f64>().expect("Longitude not an f64?");
        let latitude: f64 = centroid_str_parts[1].parse::<f64>().expect("Longitude not an f64?");
        let population: u32 = obj.get("population").expect("No population").as_u32().expect("population not an u32?");
        let geoid: String = String::from(obj.get("geoid").expect("No geoid").as_str().expect("Geoid is not a string?"));
        let state: u8 = obj.get("state").expect("No state").as_str().expect("State is not a string?").parse::<u8>().expect("State couldn't parse to u8");
        let coordinate: Coordinate = Coordinate {
            longitude,
            latitude
        };
        return CountyData {
            coordinate,
            index: 0, // index will be set later
            geoid,
            state,
            population
        };
    }
    else {
        panic!("Got unrecognized type");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn find_distance_between_london_and_paris() {
        let paris = Coordinate {
            longitude: -2.34880_f64,
            latitude: 48.85341_f64
        };
        let london = Coordinate {
            longitude: -0.12574_f64,
            latitude: 51.50853_f64
        };
        assert_eq!(335, find_distance_between_coordinates(&paris, &london).round() as u32);
    }

    #[test]
    fn find_closest_location_to_all_counties_same_population() {
        let county_data_left = make_simple_county_data(-5.0, 0.0, 1000);
        let county_data_center = make_simple_county_data(0.0, 0.0, 1000);
        let county_data_right = make_simple_county_data(5.0, 0.0, 1000);

        let expected = vec!(county_data_center.coordinate);
        let mut counties = vec!(county_data_left, county_data_center, county_data_right);
        update_county_indices(&mut counties);
        let closest = find_closest_location_to_all_counties(&counties, 1);
        assert_eq!(expected, closest);
    }

    #[test]
    fn find_closest_location_to_all_counties_different_population() {
        let county_data_left = make_simple_county_data(-5.0, 0.0, 1000);
        let county_data_center = make_simple_county_data(0.0, 0.0, 1000);
        let county_data_right = make_simple_county_data(5.0, 0.0, 5000000);

        let expected = vec!(county_data_right.coordinate);
        let mut counties = [county_data_left, county_data_center, county_data_right];
        update_county_indices(&mut counties);
        let closest = find_closest_location_to_all_counties(&counties, 1);
        assert_eq!(expected, closest);
    }

    #[test]
    fn find_closest_2_locations_to_all_counties_same_population() {
        let county_data_left_1 = make_simple_county_data(-5.0, 0.0, 1000);
        let county_data_center_1 = make_simple_county_data(0.0, 0.0, 1000);
        let county_data_right_1 = make_simple_county_data(5.0, 0.0, 1000);
        let county_data_left_2 = make_simple_county_data(25.0, 0.0, 1000);
        let county_data_center_2 = make_simple_county_data(30.0, 0.0, 1000);
        let county_data_right_2 = make_simple_county_data(35.0, 0.0, 1000);

        let expected = vec!(county_data_center_1.coordinate, county_data_center_2.coordinate);
        let mut counties = vec!(county_data_left_1, county_data_center_1, county_data_right_1, county_data_left_2, county_data_center_2, county_data_right_2);
        update_county_indices(&mut counties);
        let closest = find_closest_location_to_all_counties(&counties, 2);
        assert_eq!(expected, closest);
    }

    #[test]
    fn find_closest_population_with_2_split_locations_different_population() {
        let county_data_left_1 = make_simple_county_data(-5.0, 0.0, 1000);
        let county_data_center_1 = make_county_data_with_geoid(0.0, 0.0, 2000, "L");
        let county_data_right_1 = make_simple_county_data(5.0, 0.0, 8000);
        let county_data_left_2 = make_simple_county_data(25.0, 0.0, 4000);
        let county_data_center_2 = make_county_data_with_geoid(30.0, 0.0, 16000, "R");
        let county_data_right_2 = make_simple_county_data(35.0, 0.0, 32000);

        let expected = vec!(11000, 52000);
        let mut counties = vec!(county_data_left_1, county_data_center_1, county_data_right_1, county_data_left_2, county_data_center_2, county_data_right_2);
        update_county_indices(&mut counties);
        let closest_populations = count_closest_population(&counties, &["L", "R"]);
        assert_eq!(expected, closest_populations);
    }


    #[test]
    fn find_closest_location_with_real_data() {
        let county_datas = read_county_data();
        let closest_vec = find_closest_location_to_all_counties(&county_datas, 1);
        assert_eq!(1, closest_vec.len());
        let closest = closest_vec[0];
        // This is cheating a little bit, but it's really fast to run and a good sanity check
        assert_approx_eq!(-99.89793552425651, closest.longitude);
        assert_approx_eq!(38.08749756724239, closest.latitude);
    }
 
    fn make_simple_county_data(longitude: f64, latitude: f64, population: u32) -> CountyData {
        return CountyData {
            coordinate: Coordinate {
                longitude,
                latitude
            },
            index: 0,
            geoid: "".to_string(),
            state: 1,
            population
        };
    }

    fn make_county_data_with_geoid(longitude: f64, latitude: f64, population: u32, geoid: &str) -> CountyData {
        return CountyData {
            coordinate: Coordinate {
                longitude,
                latitude
            },
            index: 0,
            geoid: geoid.to_string(),
            state: 1,
            population
        };
    }
}
