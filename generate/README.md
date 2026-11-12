# Data generation

### For generating all the possible patterns from regex (like the `preprocessed-patterns.json`)
For Avro Phonetic, `upodesh` needs the all possible Bangla character patterns for Avro Phonetic patterns.

```
cargo r -- explode ../data/source-regex-patterns.json ../data/preprocessed-patterns.json
```

### For generating the Khipro patterns (`khipro-preprocessed-patterns.json`)
Khipro is a compositional input method, so its patterns are derived by running the
`bn-khipro-classic.mim` input method (with the `m17n-im` engine) instead of from regexes.
The exact patterns are then widened to what a user may intend while typing (like the Avro
patterns): a key without a modifier also covers its modifier family (`t` → ত/ট/ঠ), short
vowels cover the long ones (`i` → ই/ঈ), `s`/`sh`, `j`/`z` and `x`/`ng` cover each other and
consonants may or may not be joined (`mr` → ম্র/মর). Keys typed with a modifier stay exact.
Patterns whose output doesn't occur in any word of `data/source-words.txt` are skipped.

```
cargo r --release -- khipro ../data/khipro-classic/bn-khipro-classic.mim ../data/khipro-preprocessed-patterns.json
```

### For generating the FSTs (`words.fst` and the patterns FSTs of Avro and Khipro)
```
cargo r
```
