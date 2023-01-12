use windres::Build;

fn main() {
    Build::new().compile("data/assets/windows/cardgetter.rc").unwrap();
}
