extern crate qqwry;

use std::net::Ipv4Addr;
use std::str::FromStr;

pub fn main() {
    match qqwry::QQWryData::new("qqwry.dat") {
        Ok(qqwry_data) => {
            println!("data file size is {}", qqwry_data.cache_size());

            for ip_s in &["255.82.195.240", "58.83.178.16"] {
                let ip = Ipv4Addr::from_str(ip_s).unwrap();
                println!("Query: {}", ip);

                if let Some(res) = qqwry_data.query(ip) {
                    println!("Result: {} | {}", res.country, res.area);
                } else {
                    println!("Failed!");
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
