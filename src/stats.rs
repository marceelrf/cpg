use bio::io::fasta;
use std::path::Path;

fn seq_length(sequence: &str) -> i32 {
    sequence.len();
}

fn CG_count(sequence: &str) -> i32 {
    sequence.chars().filter(|&c| c == 'G' || c == 'C').count();
}

fn GC_percent(sequence: &str) -> i32 {
    sequence.chars().filter(|&c| c == 'G' || c == 'C').count();
}

fn main() {

    let fasta_path = Path::new("input.fasta");

    let mut reader = fasta::Reader::new(&fasta_path)
                        .expect("Coudn't find the path to file.");


}