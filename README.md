# upodesh
`upodesh` (In Bangla, `উপদেশ` is a synonym of the word `পরামর্শ` which means `suggestion`) is a Bangla word suggestion library.

This implementation uses an approach based on the Trie data structure which is substantially faster than the Regular Expression based approach. This is a Rust port of the Go project [`libavrophonetic`](https://github.com/mugli/libavrophonetic/) of Mehdi Hasan Khan.

## Benchmarks
`upodesh` is around **2x** faster than a heavily optimized regex based search approach.
```
upodesh a               time:   [93.330 µs 93.555 µs 93.816 µs]
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe

upodesh arO             time:   [112.73 µs 113.31 µs 113.92 µs]

upodesh bistari         time:   [10.371 µs 10.401 µs 10.430 µs]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe

regex a                 time:   [193.08 µs 193.29 µs 193.53 µs]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

regex arO               time:   [248.22 µs 249.28 µs 250.45 µs]
Found 14 outliers among 100 measurements (14.00%)
  8 (8.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe

regex bistari           time:   [355.56 µs 356.39 µs 357.26 µs]
Found 11 outliers among 100 measurements (11.00%)
  7 (7.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
```

## Acknowledgement
* [Mehdi Hasan Khan](https://github.com/mugli) and [Tahmid Sadik](https://github.com/tahmidsadik/) for their [`libavrophonetic`](https://github.com/mugli/libavrophonetic/) project.
