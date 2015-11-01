extern crate qqwry;
extern crate rand;

use std::net::Ipv4Addr;
use rand::Rng;

pub fn main() {
    match qqwry::QQWryData::new("qqwry.dat") {
        Ok(qqwry_data) => {
            println!("data file size is {}", qqwry_data.cache_size());

            for _ in [0; 65536].iter() {
                let mut rng = rand::thread_rng();
                let ip = Ipv4Addr::from(rng.gen::<u32>());
                println!("Query: {}", ip);
                if let Some(res) = qqwry_data.query(ip) {
                    println!("Result: {} | {}", res.country, res.area);
                }
                else {
                    println!("Failed!");
                }
            }
        },
        Err(e) => println!("Error: {}", e),
    }
}
