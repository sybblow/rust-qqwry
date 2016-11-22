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

macro_rules! opt_try {
    ($e:expr) => (match $e { Some(e) => e, None => return None })
}

impl QQWryData {
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<QQWryData> {
        let mut cache: Vec<u8> = Vec::new();
        try!(try!(File::open(path)).read_to_end(&mut cache));

        Ok(QQWryData { cache: cache })
    }

    pub fn query(&self, ip_addr: Ipv4Addr) -> Option<IpGeoInfo> {
        let idx_found = self.find_index(ip_addr);
        let tmp_range = &self.cache[(idx_found + 4)..];
        let record_offset = read_u24(tmp_range) as usize;

        let country: String;
        // Read country information
        let mut search_range = &self.cache[(record_offset + 4)..];
        match read_u8(search_range) {
            0x01 => {
                let country_offset = read_u24(&search_range[1..]) as usize;
                search_range = &self.cache[country_offset..];
                match read_u8(search_range) {
                    0x02 => {
                        search_range = self.jump_by_lookaside(search_range);
                        country = opt_try!(get_gbk_cstring(search_range));
                        search_range = &self.cache[(country_offset + 4)..];
                    }
                    _ => {
                        let cstr = opt_try!(get_cstring_bytes(search_range));
                        let len = cstr.len() + 1;
                        country = opt_try!(decode_gbk_bytes(cstr));
                        search_range = &search_range[len..];
                    }
                }
            }
            0x02 => {
                search_range = self.jump_by_lookaside(search_range);
                country = opt_try!(get_gbk_cstring(search_range));
                // Skip 4 bytes ip and 4 bytes country offset
                search_range = &self.cache[(record_offset + 8)..];
            }
            _ => {
                let cstr = opt_try!(get_cstring_bytes(search_range));
                let len = cstr.len() + 1;
                country = opt_try!(decode_gbk_bytes(cstr));
                search_range = &search_range[len..];
            }
        }

        let area: String;
        // Read area information
        match read_u8(search_range) {
            0x00 => {
                area = "".to_string();
            }
            0x01 | 0x02 => {
                search_range = self.jump_by_lookaside(search_range);
                area = opt_try!(get_gbk_cstring(search_range));
            }
            _ => {
                area = opt_try!(get_gbk_cstring(search_range));
            }
        }

        Some(IpGeoInfo {
            country: country,
            area: area,
        })
    }

    fn find_index(&self, ip_addr: Ipv4Addr) -> usize {
        let ip_addr = u32::from(ip_addr);

        let idx_first = read_u32(&self.cache[..]);
        let idx_last = read_u32(&self.cache[4..]);

        let mut idx_found: u32 = idx_last;
        let mut h = (idx_last - idx_first) / 7;
        let mut l = 0;

        while l <= h {
            let m = (l + h) / 2;
            let tmp_range = &self.cache[(idx_first + m * 7) as usize..];
            if ip_addr < read_u32(tmp_range) {
                h = m - 1;
            } else if ip_addr > read_u32(&self.cache[read_u24(&tmp_range[4..]) as usize..]) {
                l = m + 1;
            } else {
                idx_found = idx_first + m * 7;
                break;
            }

        }

        idx_found as usize
    }

    #[inline]
    fn jump_by_lookaside(&self, range: &[u8]) -> &[u8] {
        &self.cache[read_u24(&range[1..]) as usize..]
    }

    #[inline]
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[inline]
fn read_u32(buf: &[u8]) -> u32 {
    unsafe { mem::transmute([buf[0], buf[1], buf[2], buf[3]]) }
}

#[inline]
fn read_u24(buf: &[u8]) -> u32 {
    unsafe { mem::transmute([buf[0], buf[1], buf[2], 0]) }
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
    get_cstring_bytes(buf).and_then(|cstr| decode_gbk_bytes(cstr))
}

#[cfg(test)]
mod test {
    use super::{read_u24, read_u32, get_gbk_cstring};

    #[test]
    fn it_works() {
        assert_eq!(read_u24(&[0, 1, 0]), 1 << 8);
        assert_eq!(read_u24(&[2, 1, 0]), 258);
        assert_eq!(read_u32(&[0, 1, 0, 1]), (1 << 8) + (1 << 24));
        assert_eq!(read_u32(&[2, 1, 0, 0]), 258);
        assert_eq!(get_gbk_cstring(&[0xc4, 0xe3, 0xba, 0xc3, 0x0]),
                   Some("你好".to_string()));
        assert_eq!(get_gbk_cstring(&[0xc4, 0xe3, 0xba, 0xc3]), None);
    }
}
