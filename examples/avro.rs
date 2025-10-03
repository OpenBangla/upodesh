use peak_alloc::PeakAlloc;
use upodesh::avro::Suggest;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

fn main() {
    let suggest = Suggest::new();

    let Some(word) = std::env::args().nth(1) else {
        eprintln!("Please provide a word");
        std::process::exit(1);
    };

    let mut suggestions = suggest.suggest(&word);

    println!("Word: {}", word);
    suggestions.sort();
    println!("Suggestions: [{}]", suggestions.join(", "));

    let current_mem = PEAK_ALLOC.current_usage_as_mb();
    println!("This program currently uses {} MB of RAM.", current_mem);
    let peak_mem = PEAK_ALLOC.peak_usage_as_mb();
    println!("The max amount that was used {} MB.", peak_mem);
}
