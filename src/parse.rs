use quick_xml::Reader;
use quick_xml::events::Event;
use mysql::{Pool, Opts};
use mysql::OptsBuilder;

// #[derive(Debug)]
pub struct StationData {
    id: String,
    name:  String,
    road_number: String,
    county_number: String,
    latitude: String,
    longitude: String,
}
#[derive(Debug)]
pub struct WeatherData {
    station_id: String,
    timestamp: String,
    road_temperature: String,
    air_temperature: String,
    air_humidity: String,
    wind_speed: String,
    wind_direction: String,

}

// Parse xml file and return station_data vector
pub fn parse_station(xmlfile: &str) -> Vec<StationData> {

    let mut xml = Reader::from_file(xmlfile).expect("Failed to open file!");
    xml.trim_text(true); //remove whitespaces
    
    let mut lat_stored = false;
    let mut long_stored = false;

    let mut station_data = Vec::new();
    let mut buf = Vec::new();

    loop {
        
        match xml.read_event(&mut buf) {
            Ok(Event::Start(e)) => match e.name() {
                    b"ns0:measurementSiteRecord" => {
                        let station = StationData {

                            id: String::new(),
                            name: String::new(),
                            road_number: String::new(),
                            county_number: String::new(),
                            latitude: String::new(),
                            longitude: String::new(),

                        };
                        station_data.push(station);
                        // Get station id
                        for a in e.attributes().with_checks(false) {
                            match a {
                                Ok(ref attr) if attr.key == b"id" => {
                                    let station = station_data.last_mut().unwrap();
                                    // Utf8 to String
                                    station.id = String::from_utf8(attr.value.clone().into_owned()).unwrap();

                                }
                                Ok(_) => (),
                                Err(_) => panic!("Failed to get station id at pos {}: {:?}", xml.buffer_position(), a),
                            }
                        }
                    }
                    b"ns0:value" => {
                        let station = station_data.last_mut().unwrap();
                        station.name = xml.read_text(e.name(), &mut Vec::new()).unwrap();
                    }                                     
                    b"ns0:roadNumber" => {
                        let station = station_data.last_mut().unwrap();
                        station.road_number = xml.read_text(e.name(), &mut Vec::new()).unwrap();
                    }
                    b"ns0:countyNumber" => {
                        let station = station_data.last_mut().unwrap();
                        station.county_number = xml.read_text(e.name(), &mut Vec::new()).unwrap();
                    }
                    // For some reason latitude and longitude coordinates are stored twice in the XML file
                    b"ns0:latitude" => {
                        if lat_stored {
                            lat_stored = false;
                        } else {
                            let station = station_data.last_mut().unwrap();
                            station.latitude = xml.read_text(e.name(), &mut Vec::new()).unwrap();
                            lat_stored = true;
                        }

                    }
                    b"ns0:longitude" => {
                        if long_stored {
                            long_stored = false;
                        } else {
                            let station = station_data.last_mut().unwrap();
                            station.longitude = xml.read_text(e.name(), &mut Vec::new()).unwrap();
                            long_stored = true;
                        }

                    }
                           
                    _ => (), // There are several other `Event`s we do not consider here

            },
            Ok(Event::Eof) => break,  
            Err(e) => panic!("Error at pos {}: {:?}", xml.buffer_position(), e),

            _ => (),
        }
        buf.clear();
    }
    // Vec<StationData>
    station_data

}

pub fn parse_weather(xmlfile: &str) -> Vec<WeatherData> {
    
    let mut xml = Reader::from_file(xmlfile).expect("Failed to open file!");
    xml.trim_text(true); //remove whitespaces
    
    let mut weather_data = Vec::new();
    let mut buf = Vec::new();

    loop {
        
        match xml.read_event(&mut buf) {
            Ok(Event::Start(e)) => match e.name() {
                    b"measurementSiteReference" => {
                        let weather = WeatherData {

                            station_id: String::new(),
                            timestamp: String::new(),
                            road_temperature: String::new(),
                            air_temperature: String::new(),
                            air_humidity: String::new(),
                            wind_speed: String::new(),
                            wind_direction: String::new(),

                        };
                        weather_data.push(weather);
                        // Get station id
                        for a in e.attributes().with_checks(false) {
                            match a {
                                Ok(ref attr) if attr.key == b"id" => {
                                    let weather = weather_data.last_mut().unwrap();
                                    // Utf8 to String
                                    weather.station_id = String::from_utf8(attr.value.clone().into_owned()).unwrap();

                                }
                                Ok(_) => (),
                                Err(_) => panic!("Failed to get station id at pos {}: {:?}", xml.buffer_position(), a),
                            }
                        }
                    }
                    b"measurementTimeDefault" => {
                        let weather = weather_data.last_mut().unwrap();
                        weather.timestamp = xml.read_text(e.name(), &mut Vec::new()).unwrap();
                    }                                     
                    b"windSpeed" => { 
                        let weather = weather_data.last_mut().unwrap();
                        weather.wind_speed = xml.read_text(e.name(), &mut Vec::new()).unwrap();
                        
                    }
                    b"directionCompass" => {
                        let weather = weather_data.last_mut().unwrap();
                        weather.wind_direction = xml.read_text(e.name(), &mut Vec::new()).unwrap();
                            
                    }
                    b"airTemperature" => {
                        let weather = weather_data.last_mut().unwrap();
                        weather.air_temperature = xml.read_text(e.name(), &mut Vec::new()).unwrap();
                        
                    }
                    // For some reason latitude and longitude coordinates are stored twice in the XML file
                    b"roadSurfaceTemperature" => {
                        
                        let weather = weather_data.last_mut().unwrap();
                        weather.road_temperature = xml.read_text(e.name(), &mut Vec::new()).unwrap();
                        
                    }
                    b"humidity" => {
                        let weather = weather_data.last_mut().unwrap();
                        weather.air_humidity = xml.read_text(e.name(), &mut Vec::new()).unwrap();
                        
                    }
                           
                    _ => (), // There are several other `Event`s we do not consider here

            },
            Ok(Event::Eof) => break,  
            Err(e) => panic!("Error at pos {}: {:?}", xml.buffer_position(), e),

            _ => (),
        }
        buf.clear();
    }
    // Vec<StationData>
    weather_data




}



pub fn insert_station_data(opts: Opts, station_data: Vec<StationData>) {

    // Create new pool connection 
    let pool = Pool::new(opts).expect("Pool failed to get opts in fn insert_station_data");

    let insert_stmt = r"INSERT INTO station_data (id, lat, lon, name, road_number, county_number) 
                                    VALUES (:id, :latitude, :longitude, :name, :road_number, :county_number)
                                    ON DUPLICATE KEY UPDATE lat=:latitude, lon=:longitude, name=:name, road_number=:road_number,
                                    county_number=:county_number;";

    for mut stmt in pool.prepare(insert_stmt).into_iter() { 
        
        for i in station_data.iter() {
            // `execute` takes ownership of `params` so we pass account name by reference.
            stmt.execute(params!{
                "id" => i.id.clone(),
                "latitude" => i.latitude.clone(),
                "longitude" => i.longitude.clone(),
                "name" => i.name.clone(),
                "road_number" => i.road_number.clone(),
                "county_number" => i.county_number.clone(),
            }).unwrap();
        }
    }
}

pub fn insert_weather_data(opts: Opts, weather_data: Vec<WeatherData>) {
    let insert_stmt = "INSERT INTO weather_data 
                        (station_id, timestamp, air_temperature, road_temperature, air_humidity, wind_speed, wind_direction) 
                        VALUES (:id, :timestamp, :air_temperature, :road_temperature, :air_humidity, :wind_speed, :wind_direction)";

}
    
// Setup connection to mysql
pub fn get_opts(user: &str, pass: &str, addr: &str, database: &str) -> Opts {
    let pass: String = ::std::env::var(pass).unwrap_or(pass.to_string());
    let port: u16 = ::std::env::var("3306").ok().map(|my_port| my_port.parse().ok().unwrap_or(3306)).unwrap_or(3306);

    let mut builder = OptsBuilder::default();
    
    builder.user(Some(user)) 
            .pass(Some(pass))
            .ip_or_hostname(Some(addr))
            .tcp_port(port)
            .db_name(Some(database));
    builder.into()
    
}

pub fn create_mysql_tables(opts: Opts) {

    let pool = Pool::new(opts).expect("Pool failed to get opts in fn create_mysql_tables");

    pool.prep_exec(r"CREATE TABLE `station_data` (
                        `id` char(20) NOT NULL,
                        `lat` float DEFAULT NULL,
                        `lon` float DEFAULT NULL,
                        `name` varchar(30) DEFAULT NULL,
                        `road_number` int(10) DEFAULT NULL,
                        `county_number` int(10) DEFAULT NULL,
                        PRIMARY KEY (`id`)
                    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 ROW_FORMAT=COMPACT;", ()).expect("Failed to create table: station_data");
    pool.prep_exec(r"CREATE TABLE `weather_data` (
                    `id` int(11) NOT NULL AUTO_INCREMENT,
                    `station_id` char(20) DEFAULT NULL,
                    `timestamp` timestamp NULL DEFAULT NULL,
                    `road_temperature` float DEFAULT NULL,
                    `air_temperature` float DEFAULT NULL,
                    `air_humidity` float DEFAULT NULL,
                    `wind_speed` float DEFAULT NULL,
                    `wind_direction` varchar(10) DEFAULT NULL,
                    PRIMARY KEY (`id`),
                    KEY `station_id` (`station_id`),
                    CONSTRAINT `weather_data_ibfk_1` FOREIGN KEY (`station_id`) REFERENCES `station_data` (`id`)
                    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 ROW_FORMAT=COMPACT;", ()).expect("Failed to create table: weather_Data");
}

