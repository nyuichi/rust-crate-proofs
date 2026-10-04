# BoundPtr invariant translation diagnostic

The probe was stopped after the first selected feature (`wrong_half_region`). Native compilation completed, but Creusot translation emitted `Cannot make "raw_vec::BoundPtr::pointer" transparent in "<raw_vec::BoundPtr as creusot_std::invariant::Invariant>::invariant" as it would call a less-visible item.` No VC was generated. The remaining seven cases were not attempted in this replay. This is a translation blocker, not a failed proof obligation.
