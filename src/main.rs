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
            sequence.push_str(&line);
        }
    }

    Ok(sequence)
}

fn find_cpg_islands(sequence: &str, island_length: usize) -> Vec<usize> {
    let mut cpg_islands = Vec::new();
    let mut count = 0;

    for (i, window) in sequence.as_bytes().windows(2).enumerate() {
        if window == b"CG" {
            count += 1;
        } else {
            if count >= island_length {
                cpg_islands.push(i - count + 1);
            }
            count = 0;
        }
    }

    if count >= island_length {
        cpg_islands.push(sequence.len() - count);
    }

    cpg_islands
}

fn write_positions_to_file(positions: &[usize], output_path: &Path) -> io::Result<()> {
    let mut file = File::create(output_path)?;
    for &position in positions {
        writeln!(file, "{}", position)?;
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

    cpg(fasta_path, output_path, island_length)?;

    println!("Ilhas CpG identificadas e salvas em {:?}", output_path);
    Ok(())
}
