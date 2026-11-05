# Version-specific conversion scope

The four unsigned LittleEndian from_slice methods are provided original bodies and generate exact identities on the verified little-endian target. Their source/body was checked independently for byteorder1.5.0; the eight-method byteorder-lite result was not assumed transferable.

The signed four are inherited defaults, not provided methods. Selecting them as impl-owned roots failed resolution (initial logs retained). Their original default source always creates an unsigned slice through unsafe from_raw_parts_mut/as_mut_ptr reinterpretation before calling the unsigned method. Concrete-default selector extraction and unsafe representation/provenance obligations have not been verified here. This is not counted as a proved identity. Float conversion methods are likewise unproved and require a separate investigation; their opposite-target branch includes raw-pointer bit reinterpretation.

No opposite-endian method or other target is covered.
