pub fn jz_hour(t: usize) -> usize {
    t >> 11
}

pub fn jz_minute(t: usize) -> usize {
    ((t) >> 5) & 63
}

pub fn jz_second(t: usize) -> usize {
    ((t) & 31) * 2
}

pub fn jz_time(h: usize, m: usize, s: usize) -> usize {
    ((h) << 11) + ((m) << 5) + (s) / 2
}

pub fn jz_year(t: usize) -> usize {
    ((t) >> 9) + 1980
}

pub fn jz_month(t: usize) -> usize {
    ((t) >> 5) & 15
}

pub fn jz_day(t: usize) -> usize {
    (t) & 31
}

pub fn jz_date(y: usize, m: usize, d: usize) -> usize {
    (((y) - 1980) << 9) + ((m) << 5) + (d)
}