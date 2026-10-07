# DeepModel comparison frontend probe

This isolated, translation-only probe reproduces the comparison-trait frontend
failure on a raw-pointer/length handle shaped like upstream `Bytes`. The
negative control deliberately omits `DeepModel`; an intermediate control adds
a `Seq<u8>` receiver model but keeps the upstream generic bounds; the positive
control adds the matching RHS `DeepModel<DeepModelTy = Seq<u8>>` bound to the
generic forwarding impls. It checks whether the frontend can normalize the
comparison trait's model projection and whether the generic signature provides
the model equality required by the stock comparison specs.

The ghost snapshot is not linked to the bytes at `ptr` and `len`; concrete
comparison bodies are placeholders, while the generic reference-forwarding
bodies mirror upstream `Bytes`. These are frontend controls only, not API proof,
a runtime model proof, or evidence that byte comparison contracts are correct.
No Why3/prover phase is run. See [RESULTS.md](RESULTS.md) for the ICE context,
outcomes, and model-link options.

Use `/workspace/bytes-proof-tools/activate.sh`, offline mode, and serialize
translations with `/tmp/itoa-creusot-proof.lock`. Run the negative control:

```sh
./translate.sh negative
```

The receiver-model/RHS-bound control should stop with an ordinary `E0277` for
the missing RHS model bound. The fully typed translation command is:

```sh
./translate.sh model-gap
./translate.sh positive
```
