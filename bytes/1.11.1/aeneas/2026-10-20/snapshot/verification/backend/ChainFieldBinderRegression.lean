import extensions.adapters_more.Generated
open Aeneas Aeneas.Std Result bytes

def bufChainExpected {Self : Type _} : (forall {U : Type _}, Self → U → Result (buf.chain.Chain Self U)) :=
  fun {U : Type _} => buf.buf_impl.Buf.chain.default

def bufChainExpectedAt {Self : Type _} : (forall {U : Type _}, Self → U → Result (buf.chain.Chain Self U)) :=
  fun {U : Type _} => @buf.buf_impl.Buf.chain.default Self U

def bufMutChainExpected {Self : Type _} : (forall {U : Type _}, Self → U → Result (buf.chain.Chain Self U)) :=
  fun {U : Type _} => buf.buf_mut.BufMut.chain_mut.default

def bufMutChainExpectedAt {Self : Type _} : (forall {U : Type _}, Self → U → Result (buf.chain.Chain Self U)) :=
  fun {U : Type _} => @buf.buf_mut.BufMut.chain_mut.default Self U
