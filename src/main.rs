use clap::{ArgGroup, Parser};
use seqcol_rs::SeqCol;
use std::path::{Path, PathBuf};

/// Compute reference sequence digests
/// according to the seqcol spec.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(group(
    ArgGroup::new("input")
    .required(true)
    .args(["fasta", "sam", "seqcol"])
))]
struct Args {
    /// Input FASTA file
    #[arg(short, long)]
    fasta: Option<PathBuf>,

    /// Input SAM/BAM file
    #[arg(short, long)]
    sam: Option<PathBuf>,

    /// Input SAM/BAM file
    #[arg(short, long)]
    seqcol: Option<PathBuf>,
}

fn process_fasta<P: AsRef<Path>>(fasta_path: P) -> anyhow::Result<String> {
    let sc = SeqCol::try_from_fasta_file(fasta_path.as_ref())?;
    let d = sc.digest(seqcol_rs::DigestConfig::default())?;
    Ok(d)
}

fn process_seqcol<P: AsRef<Path>>(seqcol_path: P) -> anyhow::Result<String> {
    let sf = std::fs::File::open(seqcol_path.as_ref())?;
    let r = std::io::BufReader::new(sf);
    let val = serde_json::from_reader(r)?;
    let sc = SeqCol::try_from_seqcol(&val)?;
    let d = sc.digest(seqcol_rs::DigestConfig::default())?;
    Ok(d)
}

fn process_sam<P: AsRef<Path>>(sam_path: P) -> anyhow::Result<String> {
    let mut reader = noodles_util::alignment::io::reader::Builder::default()
        .build_from_path(sam_path.as_ref())?;
    let header = reader.read_header()?;
    let sc = SeqCol::from_sam_header(
        header
            .reference_sequences()
            .iter()
            .map(|(k, v)| (k.as_slice(), v.length().into())),
    );
    let d = sc.digest(seqcol_rs::DigestConfig::default())?;
    Ok(d)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args {
        Args {
            fasta: Some(fasta),
            sam: None,
            seqcol: None,
        } => {
            let d = process_fasta(fasta)?;
            println!("{d}");
        }
        Args {
            fasta: None,
            sam: Some(sam),
            seqcol: None,
        } => {
            let d = process_sam(sam)?;
            println!("{d}");
        }
        Args {
            fasta: None,
            sam: None,
            seqcol: Some(seqcol),
        } => {
            let d = process_seqcol(seqcol)?;
            println!("{d}");
        }
        _ => {}
    }

    Ok(())
}
