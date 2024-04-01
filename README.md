# rsrs

`rsrs` is a utility to compute `r`eference `s`ignatures (rs) in `rust` (rs).  Specifically, it is a (currently in active development) attempt to provide a rust implementation of the [`seqcol`](https://seqcol.readthedocs.io/en/latest/) specification.  The goal is to be able to take relevant input (an appropriate set of sequences or relevant description), and to produce as output the corresponding `seqcol` object, or the digest associated with that `seqcol` object.

The goal is for the tool to be simple, flexible, and fast, so that you can compute the digest or seqcol object for various different sources efficiently and with minimal hassle. If there's a feature that you think makes sense, feel free to request it. If you think you've found a bug, please report it. If you'd like to contribute to `rsrs`, feel free to open an issue to discuss or to submit a PR!
