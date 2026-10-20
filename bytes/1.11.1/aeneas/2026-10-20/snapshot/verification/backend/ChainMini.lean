import extensions.readers.Generated
open Aeneas Aeneas.Std Result bytes

def chainExpected {Self : Type _} : (forall {U : Type _}, Self → U → Result (buf.chain.Chain Self U)) :=
  fun {U : Type _} => buf.buf_impl.Buf.chain.default

def chainExpectedAt {Self : Type _} : (forall {U : Type _}, Self → U → Result (buf.chain.Chain Self U)) :=
  fun {U : Type _} => @buf.buf_impl.Buf.chain.default Self U

def chainExpectedBare {Self : Type _} : (forall {U : Type _}, Self → U → Result (buf.chain.Chain Self U)) :=
  buf.buf_impl.Buf.chain.default

def chainMutExpected {Self : Type _} : (forall {U : Type _}, Self → U → Result (buf.chain.Chain Self U)) :=
  fun {U : Type _} => buf.buf_mut.BufMut.chain_mut.default

def chainMutExpectedAt {Self : Type _} : (forall {U : Type _}, Self → U → Result (buf.chain.Chain Self U)) :=
  fun {U : Type _} => @buf.buf_mut.BufMut.chain_mut.default Self U
