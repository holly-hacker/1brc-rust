use core::str;
use rayon::prelude::*;
use std::{collections::BTreeMap, fs::File, i64};

fn main() {
    let file_name = std::env::args().nth(1).expect("No file name given");
    let file = File::open(file_name).expect("Could not open file");

    let mmap = unsafe { memmap2::Mmap::map(&file).unwrap() };

    // find separation points
    let core_count = num_cpus::get();
    let chunk_size = mmap.len() / core_count;
    let split_points_unaligned = (0..core_count).map(|i| i * chunk_size).collect::<Vec<_>>();
    let mut split_points_aligned = split_points_unaligned
        .iter()
        .map(|&i| {
            let mut i = i;
            while mmap[i] != b'\n' {
                i += 1;
            }
            i + 1
        })
        .collect::<Vec<_>>();
    split_points_aligned.push(mmap.len());

    let all_hashmaps = split_points_aligned
        .windows(2)
        .par_bridge()
        .map(|region| {
            let mut hashmap = rustc_hash::FxHashMap::default();
            let (start, end) = (region[0], region[1]);
            let region = &mmap[start..end];

            let mut file_offset = 0;
            loop {
                let Some(line_len) = memchr::memchr(
                    b'\n',
                    &region[file_offset..][..(100 + 5 + 2).min(region.len() - file_offset)],
                ) else {
                    break;
                };

                let slice = &region[file_offset..][..line_len];
                file_offset += line_len + 1;

                let idx = slice.iter().position(|&c| c == b';').expect("find ;");
                let (left, right) = slice.split_at(idx);
                let right = &right[1..]; // skip ;

                debug_assert!(right.len() <= 5); // longest val: -99.9
                let num = FixedPointNum::parse(right);

                let data = hashmap.entry(left).or_insert_with(|| SomeData {
                    min: num,
                    max: num,
                    sum: FixedPointNum::ZERO,
                    count: 0,
                });

                data.min = data.min.min(num);
                data.max = data.max.max(num);
                data.sum = FixedPointNum(data.sum.0 + num.0);
                data.count += 1;
            }

            hashmap
        })
        .collect::<Vec<_>>();

    let hashmap = all_hashmaps
        .into_iter()
        .fold(rustc_hash::FxHashMap::default(), |mut acc, x| {
            for (k, v) in x {
                let data = acc.entry(k).or_insert_with(|| SomeData {
                    min: FixedPointNum::MAX,
                    max: FixedPointNum::MIN,
                    sum: FixedPointNum::ZERO,
                    count: 0,
                });

                data.min = data.min.min(v.min);
                data.max = data.max.max(v.max);
                data.sum = FixedPointNum(data.sum.0 + v.sum.0);
                data.count += v.count;
            }

            acc
        });

    // move into btreemap to sort by key
    let sorted = hashmap.into_iter().collect::<BTreeMap<_, _>>();

    println!("Station;Min;Max;Avg");
    for (station, data) in sorted.into_iter() {
        let name = str::from_utf8(station).unwrap();
        let avg = FixedPointNum(data.sum.0 / data.count as i64).to_f32();
        println!(
            "{};{};{};{}",
            name,
            data.min.to_f32(),
            data.max.to_f32(),
            avg
        );
    }
}

struct SomeData {
    min: FixedPointNum,
    max: FixedPointNum,
    sum: FixedPointNum,
    count: usize,
}

/// Fixed point number with 1 decimal place. Abuses the known input format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FixedPointNum(i64);

impl FixedPointNum {
    const ZERO: Self = FixedPointNum(0);
    const MIN: Self = FixedPointNum(i64::MIN);
    const MAX: Self = FixedPointNum(i64::MAX);

    fn parse(input: &[u8]) -> Self {
        let mut bytes = [0u8; 5];

        // copy input to fixed size buffer
        bytes[..input.len()].copy_from_slice(input);

        // extract if negative
        let is_negative = bytes[0] == b'-';
        bytes[0] *= !is_negative as u8;

        // extract period (will never be the first byte)
        let mut has_period = false;
        for i in 1..bytes.len() {
            let is_period = bytes[i] == b'.';
            bytes[i] *= !is_period as u8;
            has_period |= is_period;
        }

        // loop over bytes, move right on null bytes
        for x in 1..bytes.len() {
            for i in (x..bytes.len()).rev() {
                if bytes[i] == 0 {
                    bytes.swap(i - 1, i);
                }
            }
        }

        // should only have leading zeroes now
        // there should always be at least 1, since we remove potential minus signs
        debug_assert!(bytes[0] == 0);

        // sum up the bytes
        let mut num = 0;
        for i in (0..bytes.len()).rev() {
            let mul = 10i64.pow((bytes.len() - 1 - i) as u32);
            if bytes[i] != 0 {
                num += (bytes[i] - b'0') as i64 * mul;
            }
        }

        if !has_period {
            num *= 10;
        }

        if is_negative {
            num *= -1;
        }

        FixedPointNum(num)
    }

    fn to_f32(self) -> f32 {
        self.0 as f32 / 10.
    }
}

#[cfg(test)]
mod tests {
    use crate::FixedPointNum;

    #[test]
    fn test_fp_internal() {
        assert_eq!(FixedPointNum::parse(b"12").0, 120);
        assert_eq!(FixedPointNum::parse(b"12.3").0, 123);
        assert_eq!(FixedPointNum::parse(b"-12.3").0, -123);
        assert_eq!(FixedPointNum::parse(b"-99.9").0, -999);
        assert_eq!(FixedPointNum::parse(b"0").0, 0);
        assert_eq!(FixedPointNum::parse(b"0.1").0, 1);
        assert_eq!(FixedPointNum::parse(b"-0.1").0, -1);
    }
}
