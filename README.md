# 1brc-rust

This repo contains an implementation of the [1 billion row challenge](https://www.morling.dev/blog/one-billion-row-challenge/), which aims to do some processing on 1 billion rows of CSV data.

There are 3 solutions:
- `solution-naive`: An initial, mostly idiomatic solution written with performance in mind.
- `solution-naive-opt`: An optimized version of the idiomatic solution with tweaks for performance.
- `solution-parallel`: `naive-opt` converted to use multiple cores.

## Expected performance

Measured at `e78ddff692ceebd1e4a9b72f13e5186e7c6fdbed`:

|           | 5800X3D | i9-14900K |
| --------- | ------: | --------: |
| naive     | 73.4s   | 47.7s     |
| naive-opt | 32.5s   | 20.5s     |
| parallel  | 3.9s    | 1.44s     |

The code in this repo isn't hyper-optimized, I just wrote these in an evening because I was challenged to.
