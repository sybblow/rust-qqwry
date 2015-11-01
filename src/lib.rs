extern crate encoding;

use std::net::Ipv4Addr;
use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::mem;

use encoding::{Encoding, DecoderTrap};
use encoding::all::GBK;

#[derive(Debug)]
pub struct IpGeoInfo {
    pub country: String,
    pub area: String,
}

pub struct QQWryData {
    cache: Vec<u8>,
}

impl QQWryData {
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<QQWryData> {
        let mut cache: Vec<u8> = Vec::new();
        try!(try!(File::open(path)).read_to_end(&mut cache));

        Ok(QQWryData {
             cache: cache,
        })
    }

    pub fn query(&self, ip_addr: Ipv4Addr) -> Option<IpGeoInfo> {
        let ip_addr = u32::from(ip_addr);

        let mut country = None;
        let mut area = None;

        let idx_first = read_u32(&self.cache[..]);
        let idx_last = read_u32(&self.cache[4..]);
        let mut idx_found = idx_last;

        let mut h = (idx_last - idx_first) / 7;
        let mut l = 0;

        while l <= h {
             let m = (l + h) / 2;
             let subcache = &self.cache[(idx_first + m*7) as usize..];
             if ip_addr < read_u32(&subcache) {
                 h = m - 1;
             } else {
                 if ip_addr > read_u32(&self.cache[read_u24(&subcache[4..]) as usize..]) {
                     l = m + 1;
                 } else {
                     idx_found = idx_first + m * 7;
                     break;
                 }
             }
        }

        let subcache = &self.cache[(idx_found + 4) as usize ..];
        let record_offset = read_u24(subcache) as usize;
        let mut subcache = &self.cache[(record_offset + 4)..];
        match read_u8(subcache) {
            0x01 => {
                let country_offset = read_u24(&subcache[1..]) as usize;
                subcache = &self.cache[country_offset..];
                match read_u8(subcache) {
                    0x02 => {
                        subcache = self.jump_by_lookaside(subcache);
                        country = get_gbk_cstring(subcache);
                        subcache = &self.cache[(country_offset + 4)..];
                        println!("Pos 1");
                    },
                    _ => {
                        country = get_gbk_cstring(subcache);
                        if let Some(cstr) = get_cstring_bytes(subcache) {
                            let len = cstr.len() + 1;
                            println!("Len {}", len);
                            country = decode_gbk_bytes(cstr);
                            subcache = &subcache[len..];
                        }
                        else {
                            return None;
                        }
                        println!("Pos 2");
                    }
                }
            },
            0x02 => {
                subcache = self.jump_by_lookaside(subcache);
                country = get_gbk_cstring(subcache);
                /* Skip 4 bytes ip and 4 bytes country offset */
                subcache = &self.cache[(record_offset + 8)..];
                println!("Pos 3");
            },
            _ => {
                if let Some(cstr) = get_cstring_bytes(subcache) {
                    let len = cstr.len() + 1;
                    country = decode_gbk_bytes(cstr);
                    subcache = &subcache[len..];
                }
                else {
                    return None;
                }
                println!("Pos 4");
            },
        }

        /* Read area information */
        let flag = read_u8(subcache);
        println!("Flag: {}", flag);
        match flag {
            0x00 => {
                area = Some("".to_string());
                println!("Return None");
            },
            0x01 | 0x02 => {
                println!("Return by jump");
                subcache = self.jump_by_lookaside(subcache);
                area = get_gbk_cstring(subcache);
            },
            _ => {
                println!("Return just there");
                area = get_gbk_cstring(subcache);
            },
        }

        if let Some(area) = area {
            if let Some(country) = country {
                return Some(IpGeoInfo{
                    country: country,
                    area: area,
                })
            }
        }
        None
    }

    #[inline]
    fn jump_by_lookaside(&self, subcache: &[u8]) -> &[u8] {
         &self.cache[read_u24(&subcache[1..]) as usize ..]
    }

    #[inline]
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[inline]
fn read_u32(buf: &[u8]) -> u32 {
    unsafe { mem::transmute([buf[0], buf[1], buf[2], buf[3]])}
}

#[inline]
fn read_u24(buf: &[u8]) -> u32 {
    unsafe { mem::transmute([buf[0], buf[1], buf[2], 0])}
}

#[inline]
fn read_u8(buf: &[u8]) -> u32 {
    buf[0] as u32
}

#[inline]
fn get_cstring_bytes(buf: &[u8]) -> Option<&[u8]> {
    buf.iter().position(|x| *x == 0).map(|i| &buf[..i])
}

#[inline]
fn decode_gbk_bytes(bytes: &[u8]) -> Option<String> {
    GBK.decode(bytes, DecoderTrap::Replace).ok()
}

#[inline]
fn get_gbk_cstring(buf: &[u8]) -> Option<String> {
    if let Some(cstr) = get_cstring_bytes(buf) {
        decode_gbk_bytes(cstr)
    }
    else {
        None
    }
}

#[test]
fn it_works() {
    assert_eq!(read_u24(&[0, 1, 0]), 1<<8);
    assert_eq!(read_u24(&[2, 1, 0]), 258);
    assert_eq!(read_u32(&[0, 1, 0, 1]), (1<<8)+(1<<24));
    assert_eq!(read_u32(&[2, 1, 0, 0]), 258);
    assert_eq!(get_gbk_cstring(&[0xc4, 0xe3, 0xba, 0xc3, 0x0]), Some("你好".to_string()));
    assert_eq!(get_gbk_cstring(&[0xc4, 0xe3, 0xba, 0xc3]), None);
}
