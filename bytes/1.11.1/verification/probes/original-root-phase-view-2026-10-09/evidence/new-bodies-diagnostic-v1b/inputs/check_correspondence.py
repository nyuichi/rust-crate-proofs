#!/usr/bin/env python3
"""AY operational source/native joins; exact proof/Cargo receipts are separate.

This reviewed bounded mapping is not a general Rust equivalence verifier.
"""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys

ROOT=Path(__file__).resolve().parent
AX=ROOT.parent/'original-raw-suffix-drop-2026-10-09'
AX_CHECKER_SHA='f63fcc1b6382be16eff6cf31a3ea4eb05a5648c615f2d681d1069817878c0922'
PREFIX_SHA='9440ac43f7a116e4bd36b5affb449334be8872e34f9e9cb7a9dc0c923311d414'
PACKAGE='bytes-original-root-phase-view'

def require(ok,message):
    if not ok:raise RuntimeError(message)

def sha(raw):return hashlib.sha256(raw).hexdigest()

def compact(source):return re.sub(r'\s+','',source)

def load_module(path,name,pin=None):
    require(path.is_file() and not path.is_symlink(),f'missing regular module {path}')
    if pin:require(sha(path.read_bytes())==pin,f'changed module {path}')
    spec=importlib.util.spec_from_file_location(name,path);require(spec and spec.loader,'module loader unavailable')
    module=importlib.util.module_from_spec(spec);sys.modules[name]=module;spec.loader.exec_module(module);return module

def ordered(source,tokens,label):
    position=0
    for token in tokens:
        found=source.find(token,position);require(found>=0,f'{label}: missing ordered operation {token}');position=found+len(token)

def audit_source(mapping,native,ax):
    prefix=(AX/'src/promotion.rs').read_bytes();raw=(ROOT/'src/promotion.rs').read_bytes()
    require(sha(prefix)==PREFIX_SHA and raw.startswith(prefix),'AY lost its immutable AX source prefix')
    require(mapping['selected_source_sha256']==sha(raw),'mapping does not identify current source')
    appendix=raw[len(prefix):].decode();full=raw.decode()
    require(b'\n'+(ROOT/'src/root_phase_extension.rs').read_bytes()==raw[len(prefix):],'AY appendix mirror differs from selected source')
    def body(name):return compact(ax.extract_function(appendix,name)[1])
    client=body('root_phase_scope')
    ordered(client,['from_box_scoped(input)','SuffixScope::new(scope)','advance_root_view(&mutvalue,first,suffix.borrow_mut())','ifpromote{','clone_suffix_root(&value,suffix.borrow_mut())','bytes_root_peer_terminal_drop(peer,suffix.borrow_mut(),peer_receipt.borrow_mut())','advance_root_view(&mutvalue,second,suffix.borrow_mut())','chunk_root_view(&value,suffix.borrow()).to_vec()','letsaved_return=result','bytes_root_view_terminal_drop(value,suffix,receipt.borrow_mut())','saved_return'],'AY witness')
    require(client.count('advance_root_view(')==2 and client.count('clone_suffix_root(')==1 and client.count('bytes_root_peer_terminal_drop(')==1 and client.count('bytes_root_view_terminal_drop(')==1,'AY witness duplicates lifecycle operation')
    # Braces locate the runtime branch itself; a same-order bag of calls is insufficient.
    start=client.index('ifpromote{')+len('ifpromote');depth=0;end=None
    for i in range(start,len(client)):
        if client[i]=='{':depth+=1
        elif client[i]=='}':
            depth-=1
            if depth==0:end=i;break
    require(end is not None,'runtime branch is unbalanced')
    branch=client[start:end+1];continuation=client[end+1:]
    require('clone_suffix_root(' in branch and 'bytes_root_peer_terminal_drop(' in branch and 'advance_root_view(&mutvalue,second' not in branch,'peer Clone/Drop is not contained in runtime promote branch')
    require('advance_root_view(&mutvalue,second' in continuation and 'clone_suffix_root(' not in continuation,'second common advance is not after branch join')
    advance=body('advance_root_view');inc=body('inc_start_root_view')
    ordered(advance,['assert!(amount<=value.len()','inc_start_root_view(value,amount,scope)'],'common advance')
    ordered(inc,['value.len-=amount','cursor_pointer::add(value.ptr,amount,bound,lease)','value.ptr=ptr','scope.view=shifted.into_inner()'],'common pointer increment')
    require(inc.count('cursor_pointer::add(')==1 and 'value.data=' not in inc and 'value.vtable=' not in inc,'common advance mutates owner or duplicates pointer add')
    read=body('root_view_as_slice');chunk=body('chunk_root_view')
    require(chunk=='root_view_as_slice(value,scope)','common chunk no longer delegates to current view read')
    for name,value in [('advance',inc),('read',read)]:
        require('Phase::Raw(raw)=>&raw.physical' in value and 'full.borrow(&shared.root.ticket.token)' in value and 'Phase::Shared(shared)' in value,f'{name} does not select live Raw or root-ticket physical borrow')
    require(read.count('physical_projection::borrow(value.ptr,value.len,bound,region)')==1,'read is not one physical borrow at current pointer/length')
    terminal=body('bytes_root_view_terminal_drop')
    ordered(terminal,['letnative=value.vtable.drop','descriptor.base.raw_pointer()','ifaddress&1usize==0usize','even_root_view_drop_registration()','odd_root_view_drop_registration()','erased_call::invoke3(native,(&mutvalue.data,value.ptr,value.len)'],'Root terminal')
    require(terminal.count('erased_call::invoke3(')==1 and 'value.ptr.addr' not in terminal and '(scope.into_inner(),&mut**output)' in terminal,'Root terminal dispatch key or affine argument package differs')
    peer=body('bytes_root_peer_terminal_drop')
    ordered(peer,['letnative=value.vtable.drop','cursor_shared_drop_registration()','Phase::Shared(shared)=>&mutshared.cursor','erased_call::invoke3(native,(&mutvalue.data,value.ptr,value.len)'],'lexical peer adapter')
    require(peer.count('erased_call::invoke3(')==1 and '(value.original_shared.into_inner(),cursor.into_inner(),&mut**output)' in peer,'peer adapter does not pass its exact owner/cursor/output')
    for parity in ('even','odd'):
        callback=body(parity+'_root_view_drop_checked')
        ordered(callback,['suffix.scope.phase.take().unwrap()','owned_pointer::get_mut_finish(data,own)','letkind=crate::provenance_specs::pointer_addr(word)&1usize','ifkind==0usize','release_core(word.cast(),root,cursor.borrow_mut(),completion.borrow_mut())','}else{','free_raw_suffix_checked(base,offset,len'],'native kind-branch callback '+parity)
        require(callback.count('owned_pointer::get_mut_finish(')==1 and callback.count('release_core(')==1 and callback.count('free_raw_suffix_checked(')==1,'callback duplicates owned read/release/free')
        require('RootDropRemainder::Raw(raw.recovery,raw.physical)' in callback and 'RootDropRemainder::Shared(shared.root,shared.cursor)' in callback,'callback does not preserve exact phase remainder')
        require('(descriptor.into_inner(),view.into_inner(),recovery.into_inner(),physical.into_inner())' in callback,'raw callback free input does not consume exact affine resources')
        decoder='tag_specs::clear_low_bit(word,ghost!{&descriptor.base})' if parity=='even' else 'word.cast::<u8>()'
        require(decoder in callback,'callback tag decoder/cast changed')
        require('RootDropEffect::Shared(DetachedScope{cursor:cursor.into_inner()},completion.into_inner().unwrap())' in callback and 'RootDropEffect::Raw(receipt.into_inner())' in callback,'callback effect package changed')
        registration=compact(ax.extract_function(appendix,parity+'_root_view_drop_registration')[0])
        require('Ghost::conjure()' in registration,'registration body differs from generic TCB instance')
    require('typeRootViewDropInput' in compact(appendix) and '=(SuffixScope,&\'amutOption<RootDropEffect>)' in compact(appendix),'Root callback input package changed')
    client_native=native['native_audit']['client']
    return dict(status='pass',source_sha256=sha(raw),appendix_sha256=sha(raw[len(prefix):]),source_join_scope='runtime promote branch/lexical peer adapter/common continuation; phase-selected physical borrow; one consuming owned-field read followed by native kind branches; immutable-base terminal registration',native_client=client_native,immutable_AX_constructor_advance_free_helpers=True,full_original_admitted=False)

# The CLI and full proof/Cargo binding are completed after actual frozen outputs.
if __name__=='__main__':
    raise SystemExit('AY correspondence checker integration is pending native/source freeze; use explicit development checker-skip mode')
