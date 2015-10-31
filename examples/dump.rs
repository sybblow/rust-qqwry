extern crate qqwry;
use std::net::Ipv4Addr;
use std::str::FromStr;

pub fn main() {
    match qqwry::QQWryData::new("qqwry.dat") {
        Ok(qqwry_data) => {
            println!("data file size is {}", qqwry_data.cache_size());

            for ip in ["1.1.1.1", "42.81.65.59"].iter() {
                println!("Query: {}", ip);
                let ip = Ipv4Addr::from_str(ip).unwrap();
                if let Some(res) = qqwry_data.query(ip) {
                    println!("Result: {} {}", res.country, res.area);
                }
                else {
                    println!("Failed!");
                }
            }
        },
        Err(e) => println!("Error: {}", e),
    }
}
