//! Reduced tests of the standard Deref purity requirement.
use core::ops::Deref;
use creusot_std::prelude::*;

pub struct ProgramReader { byte: u8 }

#[check(terminates)]
fn program_access(reader: &ProgramReader) -> &u8 { &reader.byte }

impl Deref for ProgramReader {
    type Target = u8;
    #[cfg_attr(not(feature = "ghost_calls_program"), check(terminates))]
    #[cfg_attr(feature = "ghost_calls_program", check(ghost))]
    fn deref(&self) -> &u8 { program_access(self) }
}
