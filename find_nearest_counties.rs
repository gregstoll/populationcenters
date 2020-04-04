use std::fs;
use std::iter;
use std::fmt;
use std::time;
use json;

#[derive(Debug)]
pub struct CountyData {
    coordinate: Coordinate,
    geoid: String,
    state: u8,
    population: u32
}

impl fmt::Display for CountyData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, geoid: {}, state: {}, population: {}", self.coordinate, self.geoid, self.state, self.population)
    }
}

impl Clone for CountyData {
    fn clone(&self) -> CountyData {
        CountyData {
            coordinate: self.coordinate,
            geoid: self.geoid.clone(),
            state: self.state,
            population: self.population
        }
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
fn main() {
    let start_time = time::Instant::now();
    let county_datas = read_county_data();
    println!("Got {} counties", county_datas.len());
    println!("{}", find_closest_location_to_all_counties(&county_datas));
    println!("took {} secs", time::Instant::now().duration_since(start_time).as_secs_f32());
}

fn read_county_data() -> Vec::<CountyData> {
    let contents = fs::read_to_string("data/county_centroids.json").expect("Failed to open county_centroids");
    let county_parsed_json = json::parse(&contents).expect("Failed to parse JSON");
    let county_datas : Vec::<CountyData> =
        county_parsed_json.members().map(|value| parse_county_data(value)).filter(should_process_county).collect();
    return county_datas;
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

fn find_closest_location_to_all_counties(counties: &[CountyData]) -> &Coordinate {
    static ZERO_COORDINATE: Coordinate = Coordinate {
        longitude: 0.0,
        latitude: 0.0
    };

    let result = counties
        .iter()
        .map(|county| (find_squared_distance_to_all_counties(iter::once(&county.coordinate), counties), &county.coordinate))
        .fold((1./0. /*Inf*/, &ZERO_COORDINATE), |x, y| { if x.0 < y.0 { x } else { y }});
    return result.1;
}

fn find_squared_distance_to_all_counties<'a, I>(locations: I, counties: &'a [CountyData]) -> f64
    where
        I: Iterator<Item = &'a Coordinate> + Clone {
    // TODO - this clone() is pretty ugly
    let total = counties.iter().map(|county| find_squared_distance_to_single_county(locations.clone(), &county)).sum();
    return total;
}

fn find_squared_distance_to_single_county<'a, I>(locations: I, county: &'a CountyData) -> f64
    where
        I: Iterator<Item = &'a Coordinate> + Clone {
    let county_coordinate = &county.coordinate;
    let min_distance = locations
        .map(|location| find_distance_between_coordinates(location, &county_coordinate) * f64::from(county.population))
        .fold(1./0. /*Inf*/, f64::min);
    return min_distance * min_distance;
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

        let expected = county_data_center.coordinate;
        let counties = [county_data_left, county_data_center, county_data_right];
        let closest = find_closest_location_to_all_counties(&counties);
        assert_eq!(expected, closest.clone());
    }

    #[test]
    fn find_closest_location_to_all_counties_different_population() {
        let county_data_left = make_simple_county_data(-5.0, 0.0, 1000);
        let county_data_center = make_simple_county_data(0.0, 0.0, 1000);
        let county_data_right = make_simple_county_data(5.0, 0.0, 5000000);

        let expected = county_data_right.coordinate;
        let counties = [county_data_left, county_data_center, county_data_right];
        let closest = find_closest_location_to_all_counties(&counties);
        assert_eq!(expected, closest.clone());
    }

    fn make_simple_county_data(longitude: f64, latitude: f64, population: u32) -> CountyData {
        return CountyData {
            coordinate: Coordinate {
                longitude,
                latitude
            },
            geoid: "".to_string(),
            state: 1,
            population
        };
    }
}