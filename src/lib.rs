#[derive(Debug, Clone, Copy)]
pub struct Point {
    lat: f64,
    long: f64,
}

const safecharacters: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";

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
            result.push(safecharacters[rem as usize])
        }
    }

    result
}

#[derive(Debug, Clone)]
pub struct InvalidCharError {
    c: char,
}

fn indexByte(haystack: &[u8], needle: u8) -> Option<i64> {
    // TODO(dgryski): optimize this away from a loop
    for (i, &c) in haystack.iter().enumerate() {
        if c == needle {
            return Some(i as i64);
        }
    }
    None
}

pub fn decompress(value: &[u8]) -> Result<Vec<Point>, InvalidCharError> {
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
                return Ok(points);
            }

            let r = indexByte(safecharacters, value[index]);
            if r.is_none() {
                return Err(InvalidCharError {
                    c: value[index] as char,
                });
            }
            let b = r.unwrap();
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
}
