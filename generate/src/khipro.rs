//! Pattern generation for the Khipro Classic input method.
//!
//! Khipro is a compositional state machine, so a roman substring does not map to a fixed
//! Bangla string: vowels become kars after consonants, consecutive consonants form
//! conjuncts, the slicer (`/`) breaks them and so on. Instead of hand-writing regexes like
//! we do for Avro, we run the `.mim` file with the m17n-im engine and derive the pattern
//! table from its output:
//!
//! 1. Every key of the relevant maps becomes a pattern with all the forms it can produce
//!    in isolation (and, for vowels, after a consonant).
//! 2. Multi-token sequences are converted by the engine and compared against what the
//!    suggestion algorithm would produce by greedily segmenting them with the patterns we
//!    already have. Whenever the context changes the result (`kt` → ক্ত, `ksh` → কশ, ...)
//!    the whole sequence is added as a pattern of its own.
//! 3. The exact patterns are widened to what a user may intend while typing, like the Avro
//!    patterns are: a token without a modifier also stands for its modifier family
//!    (`t` → ত/ট/ঠ), short vowels for the long ones (`i` → ই/ঈ), a few similar sounding
//!    keys for each other (`s`/`sh`, `j`/`z`, `x`/`ng`), and consonants may or may not be
//!    joined (`mr` → ম্র/মর). Tokens typed with a modifier stay exact.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs::{File, read_to_string},
    ops::Bound,
    path::Path,
};

use m17n_im::{
    Engine, ParsedInputMethod, Parser, Preprocessor,
    ast::KeySeq,
};

use crate::Block;

/// Maps whose keys produce vowels (full vowels or kars).
const VOWEL_MAPS: [&str; 3] = ["shor", "kar", "fkar"];
/// Maps whose keys produce consonants.
const CONSONANT_MAPS: [&str; 2] = ["byanjon", "phola"];
/// Maps whose keys produce other characters, independent of the context.
const OTHER_MAPS: [&str; 2] = ["diacritic", "ongko"];
/// Maps of the slicer (`/`, `//`, `///`), which breaks conjuncts or gives a chandrabindu.
const SLICER_MAPS: [&str; 3] = ["slicer", "slicer2", "slicer3"];

/// Placeholder character the input method emits for a bare modifier key.
const PLACEHOLDER: char = '◌';

/// Maximum number of tokens in a generated context pattern.
const MAX_TOKENS: usize = 4;

/// Answers whether a string occurs inside any of the dictionary words.
///
/// A pattern whose output never occurs in a word can't take part in a suggestion, so we
/// don't need to generate it.
struct Dictionary {
    suffixes: BTreeSet<String>,
}

impl Dictionary {
    fn new(words: &str) -> Self {
        let suffixes = words
            .lines()
            .map(str::trim)
            .flat_map(|word| word.char_indices().map(move |(i, _)| word[i..].to_owned()))
            .collect();

        Dictionary { suffixes }
    }

    fn contains_substring(&self, s: &str) -> bool {
        self.suffixes
            .range::<str, _>((Bound::Included(s), Bound::Unbounded))
            .next()
            .is_some_and(|suffix| suffix.starts_with(s))
    }
}

struct Converter {
    engine: Engine<ParsedInputMethod>,
    cache: HashMap<String, String>,
}

impl Converter {
    fn convert(&mut self, input: &str) -> &str {
        if !self.cache.contains_key(input) {
            let output = self.engine.convert(input);
            self.cache.insert(input.to_owned(), output);
        }

        &self.cache[input]
    }
}

/// Is the output usable as a Bangla pattern?
fn is_bangla(output: &str) -> bool {
    !output.is_empty() && !output.contains(PLACEHOLDER) && !output.chars().any(|c| c.is_ascii())
}

/// Collect the text keys of the given maps.
fn collect_keys(file: &m17n_im::ast::InputMethodFile, maps: &[&str]) -> BTreeSet<String> {
    file.maps
        .iter()
        .filter(|map| maps.contains(&file.interner.resolve(map.name)))
        .flat_map(|map| &map.rules)
        .filter_map(|rule| match &rule.key_seq {
            KeySeq::Text(key) => Some(key),
            KeySeq::Events(_) => None,
        })
        // Only keys that the suggestion input can contain
        .filter(|key| {
            key.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, ',' | '`' | '/'))
        })
        // Rules that don't insert anything (only state changes) are not patterns
        .filter(|key| !key.is_empty())
        .cloned()
        .collect()
}

/// Find the longest key which is a prefix of `input`, like the suggestion algorithm does.
fn longest_key<'a>(patterns: &BTreeMap<String, Block>, input: &'a str) -> Option<&'a str> {
    (1..=input.len())
        .rev()
        .map(|i| &input[..i])
        .find(|prefix| patterns.contains_key(*prefix))
}

/// All the Bangla strings the suggestion algorithm could build for `input` by greedily
/// segmenting it with the patterns.
///
/// Like the dictionary walk of the suggestion algorithm, only the strings occurring in the
/// dictionary words are kept.
fn segment_products(
    patterns: &BTreeMap<String, Block>,
    dictionary: &Dictionary,
    input: &str,
) -> HashSet<String> {
    let mut products: HashSet<String> = HashSet::new();
    let mut remaining = input;
    let mut first = true;

    while !remaining.is_empty() {
        let Some(key) = longest_key(patterns, remaining) else {
            return HashSet::new();
        };
        remaining = &remaining[key.len()..];

        let block = &patterns[key];

        if first {
            products = block.transliterate.iter().cloned().collect();
            first = false;
        } else {
            let mut extended: HashSet<String> = products
                .iter()
                .flat_map(|p| block.transliterate.iter().map(move |t| format!("{p}{t}")))
                .collect();

            if block.entire_block_optional.is_some() {
                extended.extend(products);
            }

            products = extended;
        }

        products.retain(|p| dictionary.contains_substring(p));
    }

    products
}

/// Build the pattern for a single token.
fn token_block(converter: &mut Converter, token: &str, vowel: bool) -> Option<Block> {
    let mut alternatives = BTreeSet::new();
    let mut optional = false;

    let standalone = converter.convert(token).to_owned();
    if is_bangla(&standalone) {
        alternatives.insert(standalone);
    }

    if vowel {
        // The form a vowel takes after a consonant (kar)
        let consonant = converter.convert("k").to_owned();
        let after_consonant = converter.convert(&format!("k{token}")).to_owned();

        if let Some(kar) = after_consonant.strip_prefix(&consonant) {
            if kar.is_empty() {
                optional = true;
            } else if is_bangla(kar) {
                alternatives.insert(kar.to_owned());
            }
        }
    }

    if alternatives.is_empty() {
        return None;
    }

    Some(Block {
        transliterate: alternatives.into_iter().collect(),
        entire_block_optional: optional.then_some(true),
    })
}

/// A candidate sequence of tokens for which we check whether it needs its own pattern.
struct Candidate {
    text: String,
    /// The tokens of the text, as the input method reads them.
    parts: Vec<String>,
}

impl Candidate {
    fn new(parts: &[&str], tokens: &BTreeSet<String>) -> Self {
        let text = parts.concat();
        // The input method matches the longest token first, so `n` + `g` is read as `ng`
        let parts = tokenize(&text, tokens)
            .unwrap_or_else(|| parts.iter().map(|s| s.to_string()).collect());

        Candidate { text, parts }
    }
}

/// Split `input` into tokens, longest token first.
fn tokenize(input: &str, tokens: &BTreeSet<String>) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        let token = (1..=remaining.len())
            .rev()
            .map(|i| &remaining[..i])
            .find(|prefix| tokens.contains(*prefix))?;
        parts.push(token.to_owned());
        remaining = &remaining[token.len()..];
    }

    Some(parts)
}

fn sort_candidates(candidates: &mut Vec<Candidate>) {
    // Shorter strings first, so their patterns are settled before the longer ones are checked
    candidates.sort_by(|a, b| a.text.len().cmp(&b.text.len()).then(a.text.cmp(&b.text)));
    candidates.dedup_by(|a, b| a.text == b.text);
}

/// Extend the candidates by every follower.
fn extend_candidates(
    candidates: &[&Candidate],
    followers: &[&str],
    tokens: &BTreeSet<String>,
) -> Vec<Candidate> {
    let mut extended: Vec<Candidate> = candidates
        .iter()
        .flat_map(|c| {
            followers.iter().map(move |f| {
                let mut parts: Vec<&str> = c.parts.iter().map(String::as_str).collect();
                parts.push(f);
                Candidate::new(&parts, tokens)
            })
        })
        .collect();

    sort_candidates(&mut extended);
    extended
}

/// Check every candidate and add the ones the patterns can't produce yet as patterns.
///
/// Returns the candidates that were added.
fn close_over(
    converter: &mut Converter,
    dictionary: &Dictionary,
    patterns: &mut BTreeMap<String, Block>,
    candidates: &[Candidate],
) -> Vec<usize> {
    let mut added = Vec::new();

    // Adding a pattern changes the segmentation of longer strings, so repeat until stable.
    loop {
        let mut changed = false;

        for (index, candidate) in candidates.iter().enumerate() {
            if patterns.contains_key(&candidate.text) {
                continue;
            }

            let output = converter.convert(&candidate.text);
            if !is_bangla(output) || !dictionary.contains_substring(output) {
                continue;
            }

            if !segment_products(patterns, dictionary, &candidate.text).contains(output) {
                let output = output.to_owned();
                patterns.insert(
                    candidate.text.clone(),
                    Block {
                        transliterate: vec![output],
                        entire_block_optional: None,
                    },
                );
                added.push(index);
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    added
}

/// Spellings a user may intend by typing a token, besides its modifier (`f`) family.
///
/// Applied transitively together with the modifier families.
const CONFUSABLE: &[(&str, &[&str])] = &[
    // Vowel length: ই/ঈ, উ/ঊ
    ("i", &["ii"]),
    ("u", &["uu"]),
    // Different keys for similar sounds: স/শ, জ/য, ং/ঙ
    ("s", &["sh"]),
    ("sh", &["s"]),
    ("j", &["z"]),
    ("z", &["j"]),
    ("x", &["ng"]),
    ("ng", &["x"]),
    // Khanda ta: ত/ৎ
    ("t", &["t/"]),
];

/// Khipro's modifier key, which turns a letter into a related one (ত → ট → ঠ, ন → ণ, ...).
const MODIFIER: char = 'f';

/// The tokens a user may have intended by typing `token` without a modifier.
///
/// A token which already has a modifier stays exact, so that typing more narrows down
/// the suggestions.
fn token_variants(
    token: &str,
    consonants: &BTreeSet<String>,
    exact: &BTreeMap<String, Block>,
) -> Vec<String> {
    let mut variants = BTreeSet::new();
    let mut queue = vec![token.to_owned()];

    while let Some(current) = queue.pop() {
        if !exact.contains_key(&current) || !variants.insert(current.clone()) {
            continue;
        }

        if consonants.contains(&current) && !current.ends_with(MODIFIER) {
            let mut modified = format!("{current}{MODIFIER}");
            while exact.contains_key(&modified) {
                queue.push(modified.clone());
                modified.push(MODIFIER);
            }
        }

        if let Some((_, others)) = CONFUSABLE.iter().find(|(c, _)| *c == current) {
            queue.extend(others.iter().map(|s| s.to_string()));
        }
    }

    variants.into_iter().collect()
}

/// Makes the fuzzy alternatives, i.e. what a user may intend by typing a sequence of tokens.
struct Fuzzer<'a> {
    exact: &'a BTreeMap<String, Block>,
    dictionary: &'a Dictionary,
    variants: HashMap<String, Vec<String>>,
    /// Fuzzy blocks of the single tokens.
    singles: BTreeMap<String, Block>,
}

impl<'a> Fuzzer<'a> {
    fn new(
        exact: &'a BTreeMap<String, Block>,
        dictionary: &'a Dictionary,
        tokens: &BTreeSet<String>,
        consonants: &BTreeSet<String>,
    ) -> Self {
        let tokens: Vec<&String> = tokens.iter().filter(|t| exact.contains_key(*t)).collect();

        let variants: HashMap<String, Vec<String>> = tokens
            .iter()
            .map(|t| (t.to_string(), token_variants(t, consonants, exact)))
            .collect();

        let singles = tokens
            .into_iter()
            .map(|token| {
                let transliterate: BTreeSet<String> = variants[token]
                    .iter()
                    .flat_map(|v| &exact[v].transliterate)
                    .filter(|t| dictionary.contains_substring(t))
                    .cloned()
                    .collect();

                // Every token stays a pattern (e.g. `6` → ৬), otherwise the suggestion
                // algorithm would stop at it and suggest words for the input before it.
                let block = if transliterate.is_empty() {
                    exact[token].clone()
                } else {
                    Block {
                        transliterate: transliterate.into_iter().collect(),
                        entire_block_optional: exact[token].entire_block_optional,
                    }
                };

                (token.clone(), block)
            })
            .collect();

        Fuzzer {
            exact,
            dictionary,
            variants,
            singles,
        }
    }

    /// Everything the user may intend by typing the tokens.
    fn alternatives(&self, parts: &[String]) -> BTreeSet<String> {
        let mut alternatives = BTreeSet::new();

        // Joined: what the input method makes of every variant of the tokens,
        // e.g. `kt` → ক্ত, ক্ট
        let mut combinations = vec![String::new()];
        for part in parts {
            // A token without a pattern of its own (e.g. `///`) only stands for itself
            let itself = [part.clone()];
            let variants = self.variants.get(part).map_or(&itself[..], Vec::as_slice);
            combinations = combinations
                .iter()
                .flat_map(|c| {
                    variants
                        .iter()
                        // A variant ending in a slicer (`t` → `t/`) followed by a typed slicer
                        // would read as a different slicer key (`t//`), which wasn't typed
                        .filter(move |v| !(c.ends_with('/') && v.starts_with('/')))
                        .map(move |v| format!("{c}{v}"))
                })
                .collect();
        }

        for combination in combinations {
            alternatives.extend(segment_products(self.exact, self.dictionary, &combination));
        }

        // Unjoined: the tokens on their own, as if the user missed an `o` or a slicer,
        // e.g. `mr` → মর
        let mut unjoined: Option<HashSet<String>> = None;
        for part in parts {
            let Some(block) = self.singles.get(part) else {
                unjoined = Some(HashSet::new());
                break;
            };

            unjoined = Some(match unjoined {
                None => block.transliterate.iter().cloned().collect(),
                Some(products) => {
                    let mut extended: HashSet<String> = products
                        .iter()
                        .flat_map(|p| block.transliterate.iter().map(move |t| format!("{p}{t}")))
                        .filter(|p| self.dictionary.contains_substring(p))
                        .collect();

                    if block.entire_block_optional.is_some() {
                        extended.extend(products);
                    }

                    extended
                }
            });
        }
        alternatives.extend(unjoined.unwrap_or_default());

        alternatives
    }
}

/// Check every candidate and add the ones whose fuzzy alternatives the patterns can't
/// produce yet as patterns.
///
/// Returns the candidates that were added.
fn close_over_fuzzy(
    dictionary: &Dictionary,
    patterns: &mut BTreeMap<String, Block>,
    candidates: &[Candidate],
    alternatives: &[BTreeSet<String>],
) -> Vec<usize> {
    let mut added = Vec::new();

    // Adding a pattern changes the segmentation of longer strings, so repeat until stable.
    loop {
        let mut changed = false;

        for (index, candidate) in candidates.iter().enumerate() {
            let needed = &alternatives[index];

            if needed.is_empty() || patterns.contains_key(&candidate.text) {
                continue;
            }

            let products = segment_products(patterns, dictionary, &candidate.text);

            if !needed.iter().all(|n| products.contains(n)) {
                patterns.insert(
                    candidate.text.clone(),
                    Block {
                        transliterate: needed.iter().cloned().collect(),
                        entire_block_optional: None,
                    },
                );
                added.push(index);
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    added
}

/// Build the fuzzy patterns from the exact ones.
fn fuzzy_patterns(
    fuzzer: &Fuzzer<'_>,
    dictionary: &Dictionary,
    base: &[Candidate],
    followers: &[&str],
    tokens: &BTreeSet<String>,
) -> BTreeMap<String, Block> {
    let mut patterns = fuzzer.singles.clone();

    // Pairs first, a triple is only worth checking if its leading pair can be in a word
    let pairs: HashMap<&str, bool> = base
        .iter()
        .filter(|c| c.parts.len() == 2)
        .map(|c| (c.text.as_str(), !fuzzer.alternatives(&c.parts).is_empty()))
        .collect();

    let alternatives: Vec<BTreeSet<String>> = base
        .iter()
        .map(|c| {
            let leading = c.parts.len() > 2
                && !pairs
                    .get(c.parts[..2].concat().as_str())
                    .copied()
                    .unwrap_or(true);

            if leading {
                BTreeSet::new()
            } else {
                fuzzer.alternatives(&c.parts)
            }
        })
        .collect();

    let added = close_over_fuzzy(dictionary, &mut patterns, base, &alternatives);

    // Extend the triples which needed their own pattern, as they still carry a context
    let triples: Vec<&Candidate> = added
        .iter()
        .map(|&i| &base[i])
        .filter(|c| c.parts.len() == MAX_TOKENS - 1)
        .collect();
    let extended = extend_candidates(&triples, followers, tokens);
    let alternatives: Vec<BTreeSet<String>> = extended
        .iter()
        .map(|c| fuzzer.alternatives(&c.parts))
        .collect();

    close_over_fuzzy(dictionary, &mut patterns, &extended, &alternatives);

    patterns
}

/// Report the Khipro test cases which the generated patterns can't reproduce.
fn self_check(
    name: &str,
    converter: &mut Converter,
    dictionary: &Dictionary,
    patterns: &BTreeMap<String, Block>,
    testcases: &Path,
) {
    let Ok(cases) = read_to_string(testcases) else {
        return;
    };

    let mut total = 0;
    let mut failed = 0;

    for line in cases.lines().skip(1) {
        let Some((input, expected)) = line.split_once('\t') else {
            continue;
        };

        // Only the words the suggestions can be made for
        if !input.chars().all(|c| c.is_ascii_lowercase() || c == '/') || !is_bangla(expected) {
            continue;
        }

        let output = converter.convert(input).to_owned();

        if output != expected {
            eprintln!("warning: engine converts {input:?} to {output:?}, expected {expected:?}");
        }

        // Patterns aren't generated for outputs that can't be part of a word
        if !dictionary.contains_substring(&output) {
            continue;
        }

        total += 1;

        if !segment_products(patterns, dictionary, input).contains(&output) {
            failed += 1;
            eprintln!("warning: {name} patterns can't produce {output:?} for {input:?}");
        }
    }

    eprintln!("Khipro test cases covered by {name} patterns: {}/{total}", total - failed);
}

pub fn generate_khipro_patterns(mim: &str, words: &Path, dest: &str) {
    let dictionary =
        Dictionary::new(&read_to_string(words).expect("Failed to read source words file"));
    let file = Parser::parse_from_path(mim).expect("Failed to parse the input method file");

    let vowels = collect_keys(&file, &VOWEL_MAPS);
    let consonants = collect_keys(&file, &CONSONANT_MAPS);
    let others = collect_keys(&file, &OTHER_MAPS);
    let slicers = collect_keys(&file, &SLICER_MAPS);

    let input_method = file
        .into_input_method(&mut Preprocessor::default())
        .expect("Failed to prepare the input method");

    let mut converter = Converter {
        engine: Engine::new(input_method),
        cache: HashMap::new(),
    };

    let mut patterns: BTreeMap<String, Block> = BTreeMap::new();

    // Single tokens
    for (tokens, vowel) in [(&vowels, true), (&consonants, false), (&others, false)] {
        for token in tokens {
            if patterns.contains_key(token) {
                continue;
            }

            if let Some(block) = token_block(&mut converter, token, vowel) {
                patterns.insert(token.clone(), block);
            }
        }
    }

    // The slicers give a chandrabindu after a vowel or on their own (`/`, `//`); after
    // consonants they break conjuncts or give a chandrabindu (`kh///`), which the context
    // patterns below take care of.
    let vowel = converter.convert("a").to_owned();
    for slicer in &slicers {
        let mut alternatives = BTreeSet::new();

        let after_vowel = converter.convert(&format!("a{slicer}")).to_owned();
        if let Some(chandrabindu) = after_vowel.strip_prefix(&vowel) {
            alternatives.insert(chandrabindu.to_owned());
        }
        alternatives.insert(converter.convert(slicer).to_owned());
        alternatives.retain(|a| is_bangla(a));

        if !alternatives.is_empty() {
            patterns.insert(
                slicer.clone(),
                Block {
                    transliterate: alternatives.into_iter().collect(),
                    entire_block_optional: None,
                },
            );
        }
    }

    // Only the consonants which made it into the patterns (e.g. the bare modifier `f` doesn't)
    let consonants: BTreeSet<String> = consonants
        .into_iter()
        .filter(|c| patterns.contains_key(c))
        .collect();

    // Every single token, including the slicers which only have a meaning in a context
    let tokens: BTreeSet<String> = patterns.keys().chain(&slicers).cloned().collect();

    // What can follow a consonant and change how it is written
    let followers: Vec<&str> = consonants
        .iter()
        .chain(&vowels)
        .chain(&slicers)
        .map(String::as_str)
        .collect();

    let mut base: Vec<Candidate> = Vec::new();

    // A vowel ends the context, but check the pairs anyway, so that a context rule after
    // a vowel can't be missed silently.
    for first in &vowels {
        for second in &followers {
            base.push(Candidate::new(&[first, second], &tokens));
        }
    }

    for first in &consonants {
        for second in &followers {
            base.push(Candidate::new(&[first, second], &tokens));

            // Only a consonant or a slicer keeps the conjunct context open
            if consonants.contains(*second) || slicers.contains(*second) {
                for third in &followers {
                    base.push(Candidate::new(&[first, second, third], &tokens));
                }
            }
        }
    }

    sort_candidates(&mut base);

    let added = close_over(&mut converter, &dictionary, &mut patterns, &base);

    // Extend the triples which needed their own pattern, as they still carry a context
    let triples: Vec<&Candidate> = added
        .iter()
        .map(|&i| &base[i])
        .filter(|c| c.parts.len() == MAX_TOKENS - 1)
        .collect();
    let extended = extend_candidates(&triples, &followers, &tokens);
    close_over(&mut converter, &dictionary, &mut patterns, &extended);

    // Re-verify everything with the final set of patterns
    let mut all: Vec<Candidate> = converter
        .cache
        .keys()
        .map(|text| Candidate {
            text: text.clone(),
            parts: Vec::new(),
        })
        .collect();
    sort_candidates(&mut all);
    close_over(&mut converter, &dictionary, &mut patterns, &all);

    let testcases = Path::new(mim).with_file_name("khipro-testcases.tsv");
    self_check("exact", &mut converter, &dictionary, &patterns, &testcases);
    eprintln!("Generated {} exact Khipro patterns", patterns.len());

    // Widen the exact patterns to what the user may intend
    let fuzzer = Fuzzer::new(&patterns, &dictionary, &tokens, &consonants);
    let fuzzy = fuzzy_patterns(&fuzzer, &dictionary, &base, &followers, &tokens);

    self_check("fuzzy", &mut converter, &dictionary, &fuzzy, &testcases);
    eprintln!("Generated {} fuzzy Khipro patterns", fuzzy.len());

    let file = File::create(dest).expect("Failed to create destination file");
    serde_json::to_writer_pretty(file, &fuzzy).expect("Failed to write Khipro patterns to file");
}
