use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;
use noodles::fasta::{self, repository::adapters::IndexedReader};


pub fn read_fasta_smart<P: AsRef<Path>>(fasta_path: P) -> io::Result<()> {
    let fasta_path = fasta_path.as_ref();
    let fai_path = fasta_path.with_extension(format!(
        "{}.fai",
        fasta_path.extension().unwrap_or_default().to_string_lossy()
    ));

    // Verifica se o arquivo .fai existe
    if fai_path.exists() {
        println!("Index .fai found ({:?}). Using indexed reading!", fai_path);

        // Carrega o leitor indexado do Noodles
        let mut reader = fasta::indexed_reader::Builder::default()
            .build_from_path(fasta_path)?;

        let index = fasta::fai::read(&fai_path)?;

        // Exemplo: Buscar uma região específica no genoma indexado em tempo constante
        // Formato da região: "chr1:1-100" (1-based)
        if let Some(first_record_name) = index.keys().next() {
            println!("Searching for a sequence for the contig: {}", first_record_name);
            
            // Lê o registro indexado
            let record = reader.read_record(first_record_name)?;
            println!("Record size: {} pb", record.sequence().len());
        }

    } else {
        println!("Index.fai NOT found. Using sequential streaming...");

        let mut reader = File::open(fasta_path)
            .map(BufReader::new)
            .map(fasta::Reader::new)?;

        // Iteração sequencial sobre cada registro do FASTA
        for result in reader.records() {
            let record = result?;
            let header = record.name();
            let seq_len = record.sequence().len();

            println!("Contig: {} | Size: {} pb", header, seq_len);
        }
    }

    Ok(())
}