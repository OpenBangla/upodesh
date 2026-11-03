fn main() {
    println!("cargo::rerun-if-changed=data/khipro-classic-sub/bn-khipro-classic.mim");

    use std::fs::File;

    let file =
        m17n_im::Parser::parse_from_path("data/khipro-classic-sub/bn-khipro-classic.mim").unwrap();
    file.generate(
        &mut m17n_im::Preprocessor::default(),
        &mut File::create("src/khipro/classic.rs").unwrap(),
    )
    .unwrap();
}
