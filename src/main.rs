use clap::{ArgGroup, Parser};
use seqcol_rs::SeqCol;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::info;
use tracing_subscriber::{EnvFilter, filter::LevelFilter, fmt, prelude::*};

#[derive(Debug, Clone)]
pub struct OutputConfig {
    lvl: seqcol_rs::DigestLevel,
    additional_attr: Vec<seqcol_rs::KnownAttr>,
}

fn output_attr_parser(s: &str) -> Result<seqcol_rs::KnownAttr, String> {
    match s {
        "name_length_pairs" => Ok(seqcol_rs::KnownAttr::NameLengthPairs),
        "sorted_name_length_pairs" => Ok(seqcol_rs::KnownAttr::SortedNameLengthPairs),
        "sorted_sequences" => Ok(seqcol_rs::KnownAttr::SortedSequences),
        t => Err(format!("Do not recognize additional attribute {t}")),
    }
}

fn output_level_parser(s: &str) -> Result<seqcol_rs::DigestLevel, String> {
    match s {
        "0" => Ok(seqcol_rs::DigestLevel::Level0),
        "1" => Ok(seqcol_rs::DigestLevel::Level1),
        "2" => Ok(seqcol_rs::DigestLevel::Level2),
        t => Err(format!(
            "output level {t} not valid; must be in {{0, 1, 2}}."
        )),
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

    /// Input an existing seqcol digest as a JSON file.
    /// This is useful e.g. for converting a level 2 digest to a level 0 or level 1 digest.
    #[arg(long)]
    seqcol: Option<PathBuf>,

    /// Optional output path; if provided, output will be written
    /// here rather than to stdout.
    #[arg(short, long)]
    out_path: Option<PathBuf>,

    /// A ',' separated list of additional attributes to include in the object; valid options are
    /// name_length_pairs, sorted_name_length_pairs, and sorted_sequences.
    #[arg(short='a', long, value_delimiter=',', num_args=0.., value_parser = output_attr_parser)]
    additional_attr: Vec<seqcol_rs::KnownAttr>,

    /// Level of output digest to produce, should be 0, 1 or 2 (0 can only be produced when actual
    /// sequences are available, as from a FASTA input).
    #[arg(short='l', long, default_value = "1", value_parser = output_level_parser)]
    level: seqcol_rs::DigestLevel,
}

fn write_seqcol_output(output_config: OutputConfig, mut sc: SeqCol) -> anyhow::Result<String> {
    let OutputConfig {
        lvl: level,
        additional_attr: attr,
    } = output_config;
    {
        let o = sc.digest(seqcol_rs::DigestConfig {
            level,
            additional_attr: attr,
        })?;
        Ok(serde_json::to_string_pretty(&o.to_json())?)
    }
}

fn process_fasta<P: AsRef<Path>>(
    fasta_path: P,
    output_config: OutputConfig,
) -> anyhow::Result<String> {
    let sc = SeqCol::try_from_fasta_file(fasta_path.as_ref())?;
    write_seqcol_output(output_config, sc)
}

fn process_seqcol<P: AsRef<Path>>(
    seqcol_path: P,
    output_config: OutputConfig,
) -> anyhow::Result<String> {
    let sf = std::fs::File::open(seqcol_path.as_ref())?;
    let r = std::io::BufReader::new(sf);
    let val = serde_json::from_reader(r)?;
    let sc = SeqCol::try_from_seqcol(&val)?;
    write_seqcol_output(output_config, sc)
}

fn process_sam<P: AsRef<Path>>(sam_path: P, output_config: OutputConfig) -> anyhow::Result<String> {
    #[allow(clippy::default_constructed_unit_structs)]
    let mut reader =
        noodles::bam::io::reader::Builder::default().build_from_path(sam_path.as_ref())?;
    let header = match reader.read_header() {
        Ok(hdr) => hdr,
        Err(_) => {
            info!("could not read BAM header, attempting to parse file as SAM");
            let mut reader =
                noodles::sam::io::reader::Builder::default().build_from_path(sam_path.as_ref())?;
            reader.read_header()?
        }
    };
    if header.is_empty() {
        anyhow::bail!(
            "The header appears empty or could not be parsed, and so no digest will be produced; ensure {} is a valid SAM/BAM file.",
            sam_path.as_ref().display()
        );
    }
    let sc = SeqCol::from_sam_header(
        header
            .reference_sequences()
            .iter()
            .map(|(k, v)| (k.as_slice(), v.length().into())),
    );
    write_seqcol_output(output_config, sc)
}

fn main() -> anyhow::Result<()> {
    // Check the `RUST_LOG` variable for the logger level and
    // respect the value found there. If this environment
    // variable is not set then set the logging level to
    // INFO.
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
                // we don't want to hear anything below a warning from ureq
                .add_directive("ureq=warn".parse()?),
        )
        .init();

    let args = Args::parse();

    let mut out_stream: Box<dyn Write> = match &args.out_path {
        Some(op) => std::fs::File::create(op).map(|f| Box::new(f) as Box<dyn Write>)?,
        None => Box::new(std::io::stdout()),
    };

    match args {
        Args {
            fasta: Some(fasta),
            sam: None,
            seqcol: None,
            out_path: _,
            additional_attr: attr,
            level: lvl,
        } => {
            let output_type = OutputConfig {
                lvl,
                additional_attr: attr,
            };
            let d = process_fasta(fasta, output_type)?;
            writeln!(out_stream, "{d}")?;
        }
        Args {
            fasta: None,
            sam: Some(sam),
            seqcol: None,
            out_path: _,
            additional_attr: attr,
            level: lvl,
        } => {
            let output_type = OutputConfig {
                lvl,
                additional_attr: attr,
            };
            let d = process_sam(sam, output_type)?;
            writeln!(out_stream, "{d}")?;
        }
        Args {
            fasta: None,
            sam: None,
            seqcol: Some(seqcol),
            out_path: _,
            additional_attr: attr,
            level: lvl,
        } => {
            let output_type = OutputConfig {
                lvl,
                additional_attr: attr,
            };
            let d = process_seqcol(seqcol, output_type)?;
            writeln!(out_stream, "{d}")?;
        }
        _ => {}
    }

    Ok(())
}
