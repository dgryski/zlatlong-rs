#[derive(Debug, Clone, Copy)]
pub struct Point {
    lat: f64,
    long: f64,
}

const SAFE_CHARACTERS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";

const SAFE_INDEX: &[u8] = &[
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 63, 255, 255, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 255,
    255, 255, 255, 255, 255, 255, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
    19, 20, 21, 22, 23, 24, 25, 255, 255, 255, 255, 62, 255, 26, 27, 28, 29, 30, 31, 32, 33, 34,
    35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
];

pub fn compress(points: &[Point]) -> Vec<u8> {
    // from http://msdn.microsoft.com/en-us/library/jj158958.aspx
    let mut latitude = 0i64;
    let mut longitude = 0i64;

    let mut result = Vec::<u8>::new();

    for point in points {
        // step 2
        let newlatitude = (point.lat * 100000.0).round() as i64;
        let newlongitude = (point.long * 100000.0).round() as i64;

        // step 3
        let mut dy = newlatitude - latitude;
        let mut dx = newlongitude - longitude;
        latitude = newlatitude;
        longitude = newlongitude;

        // step 4 and 5
        dy = (dy << 1) ^ (dy >> 31);
        dx = (dx << 1) ^ (dx >> 31);

        // step 6
        let mut index = (((dy + dx) * (dy + dx + 1) / 2) + dy) as i64;

        while index > 0 {
            // step 7
            let mut rem = index & 31;
            index = (index - rem) / 32;

            // step 8
            if index > 0 {
                rem += 32
            }

            // step 9
            result.push(SAFE_CHARACTERS[rem as usize])
        }
    }

    result
}

#[derive(Debug)]
pub enum DecompressError {
    InvalidCharError(char),
    TruncatedError,
}

pub fn decompress(value: &[u8]) -> Result<Vec<Point>, DecompressError> {
    // From https://docs.microsoft.com/en-us/bingmaps/spatial-data-services/geodata-api

    let mut points = Vec::<Point>::new();
    let mut index = 0;
    let mut xsum = 0;
    let mut ysum = 0;
    let max = 4294967296;

    while index < value.len() {
        let mut n = 0i64;
        let mut k = 0u64;

        loop {
            if index >= value.len() {
                return Err(DecompressError::TruncatedError);
            }

            let b = SAFE_INDEX[value[index] as usize] as i64;
            if b == 255 {
                return Err(DecompressError::InvalidCharError(value[index] as char));
            }
            index += 1;

            let tmp = (b & 31) * (1 << k);

            let ht = tmp / max;
            let lt = tmp % max;

            let hn = n / max;
            let ln = n % max;

            let nl = lt | ln;
            n = (ht | hn) * max + nl;
            k += 5;
            if b < 32 {
                break;
            }
        }

        let diagonal = (((8.0 * (n as f64) + 5.0).sqrt() - 1.0) / 2.0) as i64;

        n -= diagonal * (diagonal + 1) / 2;
        let mut ny = n;
        let mut nx = diagonal - ny;
        nx = (nx >> 1) ^ -(nx & 1);
        ny = (ny >> 1) ^ -(ny & 1);
        xsum += nx;
        ysum += ny;
        let lat = (ysum as f64) * 0.00001;
        let long = (xsum as f64) * 0.00001;
        points.push(Point { lat, long });
    }
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let output = b"vx1vilihnM6hR7mEl2Q";

        let mut points = Vec::<Point>::new();
        points.push(Point {
            lat: 35.894309002906084,
            long: -110.72522000409663,
        });
        points.push(Point {
            lat: 35.89393097907304,
            long: -110.72577999904752,
        });
        points.push(Point {
            lat: 35.89374498464167,
            long: -110.72606003843248,
        });
        points.push(Point {
            lat: 35.893366960808635,
            long: -110.72661500424147,
        });

        let r = compress(&points);

        assert_eq!(r, output);

        let p2 = decompress(&r).unwrap();

        for (i, &p) in points.iter().enumerate() {
            assert!((p.lat - p2[i].lat).abs() < 1e-5 && (p.long - p2[i].long).abs() < 1e-5);
        }
    }

    #[test]
    fn test_safe_idx_inverse() {
        for (i, &c) in SAFE_CHARACTERS.iter().enumerate() {
            // for all the characters in safecharacters, safeidx of that character is the offset
            assert!(SAFE_INDEX[c as usize] == i as u8)
        }

        for (c, &i) in SAFE_INDEX.iter().enumerate() {
            if i == 255 {
                // the character c is not in safecharacters
                let cc = &(c as u8);
                assert!(!SAFE_CHARACTERS.contains(cc));
            } else {
                // the character c has offset i in safecharacters
                assert!(SAFE_CHARACTERS[i as usize] == c as u8)
            }
        }
    }
}
