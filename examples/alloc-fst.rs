use upodesh::regex::RegexSuggest;
use peak_alloc::PeakAlloc;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

fn main() {
    let suggest = RegexSuggest::new();

    let current_mem = PEAK_ALLOC.current_usage_as_mb();
	println!("This program currently uses {} MB of RAM.", current_mem);
	let peak_mem = PEAK_ALLOC.peak_usage_as_mb();
	println!("The max amount that was used {} MB.", peak_mem);
}
