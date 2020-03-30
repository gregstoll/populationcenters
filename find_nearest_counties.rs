use std::fs;

#[derive(Debug)]
pub struct CountyData {
    longitude: f32,
    latitude: f32,
    geoid: String,
    state: u8,
    population: u32
}

fn main() {
    let contents = fs::read_to_string("data/county_centroids.json").expect("Failed to open county_centroids");
    let county_parsed_json = json::parse(&contents).expect("Failed to parse JSON");
    let county_datas : Vec::<CountyData> =
        county_parsed_json.members().map(|value| parse_county_data(value)).filter(should_process_county).collect();
    println!("Got {} counties", county_datas.len());
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

fn parse_county_data(j: &json::JsonValue) -> CountyData {
    if let json::JsonValue::Object(obj) = j {
        let centroid_str = obj.get("centroid").expect("No centroid").as_str().expect("Centroid is not a string?");
        let centroid_str_parts: Vec<&str> = centroid_str.split(',').collect();
        let longitude: f32 = centroid_str_parts[0].parse::<f32>().expect("Longitude not an f32?");
        let latitude: f32 = centroid_str_parts[1].parse::<f32>().expect("Longitude not an f32?");
        let population: u32 = obj.get("population").expect("No population").as_u32().expect("population not an u32?");
        let geoid: String = String::from(obj.get("geoid").expect("No geoid").as_str().expect("Geoid is not a string?"));
        let state: u8 = obj.get("state").expect("No state").as_str().expect("State is not a string?").parse::<u8>().expect("State couldn't parse to u8");
        return CountyData {
            longitude,
            latitude,
            geoid,
            state,
            population
        };
    }
    else {
        panic!("Got unrecognized type");
    }
}