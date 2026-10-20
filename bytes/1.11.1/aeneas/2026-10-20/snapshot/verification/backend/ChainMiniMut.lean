import extensions.adapters_more.Generated
open Aeneas Aeneas.Std Result bytes

def chainMutExpected {Self : Type _} : (forall {U : Type _}, Self → U → Result (buf.chain.Chain Self U)) :=
  fun {U : Type _} => buf.buf_mut.BufMut.chain_mut.default

def chainMutExpectedAt {Self : Type _} : (forall {U : Type _}, Self → U → Result (buf.chain.Chain Self U)) :=
  fun {U : Type _} => @buf.buf_mut.BufMut.chain_mut.default Self U
