use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

fn read_fasta(file_path: &Path) -> io::Result<String> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut sequence = String::new();

    for line in reader.lines() {
        let line = line?;
        if !line.starts_with('>') {
            sequence.push_str(&line.trim());
        }
    }

    Ok(sequence)
}

fn find_cpg_islands(sequence: &str, min_length: usize) -> Vec<(usize, usize)> {
    let mut cpg_islands = Vec::new();
    let mut start = None;

    for i in 0..=sequence.len().saturating_sub(min_length) {
        let window = &sequence[i..i + min_length];
        let gc_count = window.chars().filter(|&c| c == 'G' || c == 'C').count();
        let cpg_count = window.matches("CG").count();
        
        let gc_content = gc_count as f64 / min_length as f64;
        let expected_cpg = ((gc_count as f64 / 2.0).powi(2)) / min_length as f64;
        let cpg_ratio = cpg_count as f64 / expected_cpg;

        if gc_content >= 0.5 && cpg_ratio >= 0.6 {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start {
            cpg_islands.push((s, i + min_length - 1));
            start = None;
        }
    }

    if let Some(s) = start {
        cpg_islands.push((s, sequence.len() - 1));
    }

    cpg_islands
}

fn write_positions_to_file(positions: &[(usize, usize)], output_path: &Path) -> io::Result<()> {
    let mut file = File::create(output_path)?;
    for &(start, end) in positions {
        writeln!(file, "{} {}", start, end)?;
    }
    Ok(())
}

fn cpg(fasta_path: &Path, output_path: &Path, island_length: usize) -> io::Result<()> {
    let sequence = read_fasta(fasta_path)?;
    let cpg_islands = find_cpg_islands(&sequence, island_length);
    write_positions_to_file(&cpg_islands, output_path)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let fasta_path = Path::new("input.fasta");
    let output_path = Path::new("output.txt");
    let island_length = 200;

    let sequence = read_fasta(fasta_path)?;
    println!("{}", sequence);

    cpg(fasta_path, output_path, island_length)?;

    println!("Ilhas CpG identificadas e salvas em {:?}", output_path);
    Ok(())
}
