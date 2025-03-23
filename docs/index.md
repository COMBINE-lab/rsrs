# `rsrs`: Reference Signatures (RS) in Rust (rs)


`rsrs` is a utility to compute `r`eference `s`ignatures (rs) in `rust` (rs).  Specifically, it is a (currently in active development) attempt to provide a rust implementation of the [`seqcol`](https://seqcol.readthedocs.io/en/latest/) specification.  The goal is to be able to take relevant input (an appropriate set of sequences or relevant description), and to produce as output the corresponding `seqcol` object, or the digest associated with that `seqcol` object. This program is not part of the official seqcols project, but is designed to adhere to the spec[^seqcols].

The goal is for the tool to be simple, flexible, and fast, so that you can compute the digest or seqcol object for various different sources efficiently and with minimal hassle. If there's a feature that you think makes sense, feel free to request it. If you think you've found a bug, please report it. If you'd like to contribute to `rsrs`, feel free to open an issue to discuss or to submit a PR!

## Using `rsrs`

The `rsrs` tool takes as input a FASTA file, or a SAM/BAM file and produces a JSON object representing the corresponding seqcols object.
It can produce a level 0, 1 or 2 seqcol object (see the seqcols spec[^seqcols]) for more information on the difference between these levels.
By default, the object will contain the available *required* attributes, but you may include additional attributes using the `-a` option.
The resulting digest is printed to `stdout`, and so can easily be redirected to a JSON file if desired.

The command line usage for `rsrs` is as follows:

```sh
command line tool to compute seqcol objects and digests

Usage: rsrs [OPTIONS] <--fasta <FASTA>|--sam <SAM>|--seqcol <SEQCOL>>

Options:
      --fasta <FASTA>                           Input FASTA file
      --sam <SAM>                               Input SAM/BAM file
      --seqcol <SEQCOL>                         Input an existing seqcol digest as a JSON file. This is useful e.g. for converting a level 2 digest to a level 0 or level 1 digest
  -o, --out-path <OUT_PATH>                     Optional output path; if provided, output will be written here rather than to stdout
  -a, --additional-attr [<ADDITIONAL_ATTR>...]  A ',' separated list of additional attributes to include in the object; valid options are name_length_pairs, sorted_name_length_pairs, and
                                                sorted_sequences
  -l, --level <LEVEL>                           Level of output digest to produce, should be 0, 1 or 2 (0 can only be produced when actual sequences are available, as from a FASTA input) [default:
                                                1]
  -h, --help                                    Print help
  -V, --version                                 Print version
```

## References

[^seqcols]: [The seqcol spec](https://ga4gh.github.io/refget/seqcols/)
