//! QQwry data reader for parsing binary format

use super::utils::bytes3_to_u32;

/// Redirect mode constants
pub const REDIRECT_MODE_1: u8 = 0x01;
pub const REDIRECT_MODE_2: u8 = 0x02;

/// Reader for parsing QQwry data
pub struct Reader<'a> {
    data: &'a [u8],
    pos: u32,
    last_pos: u32,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            last_pos: 0,
        }
    }

    /// Seek to absolute position
    fn seek_abs(&mut self, offset: u32) {
        self.last_pos = self.pos;
        self.pos = offset;
    }

    /// Seek back to last position (can only call once)
    fn seek_back(&mut self) {
        self.pos = self.last_pos;
    }

    /// Read mode byte
    fn read_mode(&mut self) -> u8 {
        let mode = self.data[self.pos as usize];
        self.last_pos = self.pos;
        self.pos += 1;
        mode
    }

    /// Read 3 bytes as u32 offset
    fn read_offset(&mut self, follow: bool) -> u32 {
        let offset = bytes3_to_u32(&self.data[self.pos as usize..self.pos as usize + 3]);
        self.last_pos = self.pos;
        self.pos += 3;
        if follow {
            // Update last_pos again before jumping, matching Golang behavior
            // This ensures seekBack() returns to the position after reading the offset
            self.last_pos = self.pos;
            self.pos = offset;
        }
        offset
    }

    /// Read null-terminated string
    fn read_string(&mut self, advance: bool) -> Vec<u8> {
        let start = self.pos as usize;
        let mut end = start;
        while end < self.data.len() && self.data[end] != 0 {
            end += 1;
        }

        if advance {
            self.last_pos = self.pos;
            self.pos = (end + 1) as u32;
        }

        self.data[start..end].to_vec()
    }

    /// Parse location data at given offset
    pub fn parse(&mut self, offset: u32) -> (Vec<u8>, Vec<u8>) {
        if offset != 0 {
            self.seek_abs(offset);
        }

        let mode = self.read_mode();
        match mode {
            REDIRECT_MODE_1 => {
                // Mode 1: [IP][0x01][绝对偏移地址] - 完全重定向
                self.read_offset(true);
                self.parse(0)
            }
            REDIRECT_MODE_2 => {
                // Mode 2: [IP][0x02][国家信息的绝对偏移][地区信息]
                let country = self.parse_redirect_mode2();
                let area = self.read_area();
                (country, area)
            }
            _ => {
                // 直接存储：[IP][国家][地区]
                self.seek_back();
                let country = self.read_string(true);
                let area = self.read_area();
                (country, area)
            }
        }
    }

    /// Parse redirect mode 2 country
    fn parse_redirect_mode2(&mut self) -> Vec<u8> {
        self.read_offset(true);
        let str = self.read_string(false);
        self.seek_back();
        str
    }

    /// Read area information
    fn read_area(&mut self) -> Vec<u8> {
        let mode = self.read_mode();
        if mode == REDIRECT_MODE_1 || mode == REDIRECT_MODE_2 {
            let offset = self.read_offset(true);
            if offset == 0 {
                return Vec::new();
            }
        } else {
            self.seek_back();
        }
        self.read_string(false)
    }
}
