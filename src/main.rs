use clap::Parser;
use seqcol_rs::SeqCol;
use std::path::PathBuf;

/// Compute reference sequence digests
/// according to the seqcol spec.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input FASTA file
    #[arg(short, long)]
    fasta: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let sc = SeqCol::try_from_fasta_file(args.fasta)?;
    let d = sc.digest(seqcol_rs::DigestConfig::default())?;
    println!("{d}");

    Ok(())
}
