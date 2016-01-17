extern crate qqwry;

use std::net::Ipv4Addr;
use std::str::FromStr;

pub fn main() {
    match qqwry::QQWryData::new("qqwry.dat") {
        Ok(qqwry_data) => {
            println!("data file size is {}", qqwry_data.cache_size());

            let ip = Ipv4Addr::from_str("255.82.195.240").unwrap();
            println!("Query: {}", ip);
            if let Some(res) = qqwry_data.query(ip) {
                if res.country == " CZ88.NET" {
                    println!("保留地址");
                } else {
                    println!("Result: {} | {}", res.country, res.area);
                }
            }
            else {
                println!("Failed!");
            }
        },
        Err(e) => println!("Error: {}", e),
    }
}
