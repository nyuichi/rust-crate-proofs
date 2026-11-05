use rand_xoshiro::Seed512;
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(70)]
fn public_tuple_constructor_exact_64bytes(){let bytes:[u8;64]=kani::any();let seed=Seed512(bytes);assert_eq!(seed.0,bytes);assert_eq!(<Seed512 as AsRef<[u8]>>::as_ref(&seed),&bytes);}
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(70)]
fn constructed_mutview_exact_fullwriteback(){let bytes:[u8;64]=kani::any();let replacement:[u8;64]=kani::any();let mut seed=Seed512(bytes);let view=<Seed512 as AsMut<[u8]>>::as_mut(&mut seed);assert_eq!(view.len(),64);view.copy_from_slice(&replacement);assert_eq!(seed.0,replacement);assert_eq!(<Seed512 as AsRef<[u8]>>::as_ref(&seed),&replacement);}
