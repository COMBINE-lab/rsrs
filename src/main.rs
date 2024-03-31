use clap::{ArgGroup, Parser};
use seqcol_rs::SeqCol;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum OutputConfig {
    Digest,
    SeqColObj,
    SeqColObjSNLP,
}

fn output_config_parser(s: &str) -> Result<OutputConfig, String> {
    match s {
        "digest" => Ok(OutputConfig::Digest),
        "seqcol-obj" => Ok(OutputConfig::SeqColObj),
        "seqcol-obj-snlp" => Ok(OutputConfig::SeqColObjSNLP),
        t => Err(format!("Do not recognize output config {t}")),
    }
}

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
    #[arg(long)]
    fasta: Option<PathBuf>,

    /// Input SAM/BAM file
    #[arg(long)]
    sam: Option<PathBuf>,

    /// Input SAM/BAM file
    #[arg(long)]
    seqcol: Option<PathBuf>,

    /// Optional output path; if provided, output will be written
    /// here rather than to stdout.
    #[arg(short, long)]
    out_path: Option<PathBuf>,

    /// Type of output to produce, one of "digest", "seqcol-obj" or "seqcol-obj-snlp".
    #[arg(short='t', long, default_value = "digest", value_parser = output_config_parser)]
    output_type: OutputConfig,
}

fn process_fasta<P: AsRef<Path>>(
    fasta_path: P,
    output_config: OutputConfig,
) -> anyhow::Result<String> {
    let sc = SeqCol::try_from_fasta_file(fasta_path.as_ref())?;
    match output_config {
        OutputConfig::Digest => Ok(sc.digest(seqcol_rs::DigestConfig::default())?),
        OutputConfig::SeqColObj => {
            let o = sc.seqcol_obj(seqcol_rs::DigestConfig::default())?;
            Ok(serde_json::to_string_pretty(&o)?)
        }
        OutputConfig::SeqColObjSNLP => {
            let o = sc.seqcol_obj(seqcol_rs::DigestConfig::WithSeqnameLenPairs)?;
            Ok(serde_json::to_string_pretty(&o)?)
        }
    }
}

fn process_seqcol<P: AsRef<Path>>(
    seqcol_path: P,
    output_config: OutputConfig,
) -> anyhow::Result<String> {
    let sf = std::fs::File::open(seqcol_path.as_ref())?;
    let r = std::io::BufReader::new(sf);
    let val = serde_json::from_reader(r)?;
    let sc = SeqCol::try_from_seqcol(&val)?;
    match output_config {
        OutputConfig::Digest => Ok(sc.digest(seqcol_rs::DigestConfig::default())?),
        OutputConfig::SeqColObj => {
            let o = sc.seqcol_obj(seqcol_rs::DigestConfig::default())?;
            Ok(serde_json::to_string_pretty(&o)?)
        }
        OutputConfig::SeqColObjSNLP => {
            let o = sc.seqcol_obj(seqcol_rs::DigestConfig::WithSeqnameLenPairs)?;
            Ok(serde_json::to_string_pretty(&o)?)
        }
    }
}

fn process_sam<P: AsRef<Path>>(sam_path: P, output_config: OutputConfig) -> anyhow::Result<String> {
    let mut reader = noodles_util::alignment::io::reader::Builder::default()
        .build_from_path(sam_path.as_ref())?;
    let header = reader.read_header()?;
    let sc = SeqCol::from_sam_header(
        header
            .reference_sequences()
            .iter()
            .map(|(k, v)| (k.as_slice(), v.length().into())),
    );
    match output_config {
        OutputConfig::Digest => Ok(sc.digest(seqcol_rs::DigestConfig::default())?),
        OutputConfig::SeqColObj => {
            let o = sc.seqcol_obj(seqcol_rs::DigestConfig::default())?;
            Ok(serde_json::to_string_pretty(&o)?)
        }
        OutputConfig::SeqColObjSNLP => {
            let o = sc.seqcol_obj(seqcol_rs::DigestConfig::WithSeqnameLenPairs)?;
            Ok(serde_json::to_string_pretty(&o)?)
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut out_stream: Box<dyn Write> = match &args.out_path {
        Some(ref op) => std::fs::File::create(op).map(|f| Box::new(f) as Box<dyn Write>)?,
        None => Box::new(std::io::stdout()),
    };

    match args {
        Args {
            fasta: Some(fasta),
            sam: None,
            seqcol: None,
            out_path: _,
            output_type,
        } => {
            let d = process_fasta(fasta, output_type)?;
            writeln!(out_stream, "{d}")?;
        }
        Args {
            fasta: None,
            sam: Some(sam),
            seqcol: None,
            out_path: _,
            output_type,
        } => {
            let d = process_sam(sam, output_type)?;
            writeln!(out_stream, "{d}")?;
        }
        Args {
            fasta: None,
            sam: None,
            seqcol: Some(seqcol),
            out_path: _,
            output_type,
        } => {
            let d = process_seqcol(seqcol, output_type)?;
            writeln!(out_stream, "{d}")?;
        }
        _ => {}
    }

    Ok(())
}
