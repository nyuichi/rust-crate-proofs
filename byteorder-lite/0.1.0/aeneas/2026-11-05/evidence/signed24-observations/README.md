# Finite native signed-write observations

Released source and Cargo.toml are unchanged and byte-matched to the upstream archive. All 176 selected public calls passed the executable assertions in src/main.rs. This is concrete finite evidence, not a universal API proof.

write_i24 documents panic only on input buffer length below three, so its observed low24-bit truncation for out-of-range i32 is not itself a contradiction of that panic text. write_int documents panic for values not representable in nbytes. Actual BE/LE write_int calls with nbytes=3 and values 8388608, -8388609, i32 MIN/MAX and i64 MIN/MAX return normally when the buffer is long enough, writing the low24 bits. Buffers shorter than three panic without modification in all selected cases; successful five-byte outputs preserve both suffix bytes.

This is a documentation/observed-behavior mismatch for write_int at these inputs. It is not a demonstrated Rust memory-safety bug. These signed API observations are separate from the proved unsigned-u24 contracts in candidate-u24. Original docs/body line excerpts and source hashes are preserved.
