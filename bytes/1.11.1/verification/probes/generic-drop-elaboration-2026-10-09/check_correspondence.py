#!/usr/bin/env python3
"""Independent, closed-scope native-MIR / proof-shadow Drop correspondence checker.

The receipt is checked, never trusted as the derivation: source bodies, pinned
post-ElaborateDrops MIR, shadow call sites and copied helper bodies are reparsed.
This checker covers only this three-function straight-line probe.
"""
from __future__ import annotations
import argparse, copy, hashlib, json, pathlib, re, sys, tempfile, shutil
from dataclasses import dataclass
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
PIN_RELEASE = 'rustc 1.98.0-nightly (91fe22da8 2026-06-21)'
PIN_COMMIT = '91fe22da8084a1c9e993d78d4a56f22ab8396236'
PIN_STAGE = '2-2-004.ElaborateDrops.after.mir'
EFFECTS = [
    {'scope':'set_true_scope','type':'SetTrueOnDrop','binding':'_guard','helper':'set_true_drop_effect','mir':'generic_drop_native.set_true_scope.'+PIN_STAGE,'effects':['set_true']},
    {'scope':'toggle_scope','type':'ToggleOnDrop','binding':'_guard','helper':'toggle_drop_effect','mir':'generic_drop_native.toggle_scope.'+PIN_STAGE,'effects':[]},
    {'scope':'toggle_after_write','type':'ToggleOnDrop','binding':'guard','helper':'toggle_drop_effect','mir':'generic_drop_native.toggle_after_write.'+PIN_STAGE,'effects':[]},
]
FEATURES={'omit_drop_call','duplicate_drop_call','wrong_drop_target','wrong_drop_summary','early_drop_call'}
class AuditError(Exception):
    def __init__(self, message: str, category: str='coverage_failure'):
        super().__init__(message); self.category=category

def sha(b: bytes)->str: return hashlib.sha256(b).hexdigest()
def compact(s: str)->str: return re.sub(r'\s+',' ',s).strip()

def mask_rust_noncode(text: str)->str:
    """Mask Rust comments and string literals without changing offsets."""
    chars=list(text); i=0; n=len(text)
    def blank(a:int,b:int)->None:
        for j in range(a,b):
            if chars[j]!='\n': chars[j]=' '
    while i<n:
        if text.startswith('//',i):
            j=text.find('\n',i)
            if j<0: j=n
            blank(i,j); i=j; continue
        if text.startswith('/*',i):
            start=i; depth=1; i+=2
            while i<n and depth:
                if text.startswith('/*',i): depth+=1; i+=2
                elif text.startswith('*/',i): depth-=1; i+=2
                else: i+=1
            if depth: raise AuditError('unterminated Rust block comment')
            blank(start,i); continue
        raw=re.match(r'(?:br|rb|cr|r)(#{0,})"',text[i:])
        if raw:
            start=i; hashes=raw.group(1); i+=len(raw.group(0)); close='"'+hashes
            end=text.find(close,i)
            if end<0: raise AuditError('unterminated Rust raw string')
            i=end+len(close); blank(start,i); continue
        if text[i]=='"':
            start=i; i+=1; escaped=False
            while i<n:
                c=text[i]; i+=1
                if escaped: escaped=False
                elif c=='\\': escaped=True
                elif c=='"': break
            else: raise AuditError('unterminated Rust string')
            blank(start,i); continue
        i+=1
    return ''.join(chars)

def item_at(text: str, code: str, start: int, label: str)->tuple[int,int,str]:
    brace=code.find('{',start)
    if brace<0: raise AuditError(f'missing item body: {label}')
    depth=0; i=brace
    while i<len(code):
        c=code[i]
        if c=='{': depth+=1
        elif c=='}':
            depth-=1
            if depth==0: return start,i+1,text[start:i+1]
        i+=1
    raise AuditError(f'unclosed item: {label}')

def item_after(text: str, marker: str, *, from_pos: int=0)->tuple[int,int,str]:
    code=mask_rust_noncode(text); start=code.find(marker,from_pos)
    if start<0: raise AuditError(f'missing item: {marker}')
    return item_at(text,code,start,marker)

def body_after(text: str, marker: str)->tuple[str,int,int,str]:
    start,end,item=item_after(text,marker)
    op=mask_rust_noncode(item).find('{'); return item[op+1:-1],start,end,item

def tokens(s: str)->list[str]:
    """Small Rust token scanner: rejects unsupported structure by exact patterns."""
    out=[]; i=0; multi=('::','->','=>','==','!=','<=','>=','&&','||','+=','-=','..')
    while i<len(s):
        if s[i].isspace(): i+=1; continue
        if s.startswith('//',i):
            j=s.find('\n',i); i=len(s) if j<0 else j+1; continue
        if s.startswith('/*',i):
            depth=1; i+=2
            while i<len(s) and depth:
                if s.startswith('/*',i): depth+=1; i+=2
                elif s.startswith('*/',i): depth-=1; i+=2
                else: i+=1
            if depth: raise AuditError('unterminated Rust comment')
            continue
        if s[i]=='"':
            j=i+1; esc=False
            while j<len(s):
                if esc: esc=False
                elif s[j]=='\\': esc=True
                elif s[j]=='"': break
                j+=1
            if j>=len(s): raise AuditError('unterminated string token')
            out.append(s[i:j+1]); i=j+1; continue
        if s[i]=="'" and i+1<len(s) and (s[i+1].isalpha() or s[i+1]=='_'):
            m=re.match(r"'[A-Za-z_][A-Za-z0-9_]*",s[i:]); out.append(m.group(0)); i+=len(m.group(0)); continue
        if s[i].isalpha() or s[i]=='_':
            m=re.match(r'[A-Za-z_][A-Za-z0-9_]*',s[i:]); out.append(m.group(0)); i+=len(m.group(0)); continue
        if s[i].isdigit():
            m=re.match(r'[0-9]+',s[i:]); out.append(m.group(0)); i+=len(m.group(0)); continue
        op=next((x for x in multi if s.startswith(x,i)),None)
        if op: out.append(op); i+=len(op); continue
        out.append(s[i]); i+=1
    return out

def find_named_fn(text: str, name: str, *, visibility: str|None=None)->tuple[str,int,int,str,str]:
    code=mask_rust_noncode(text)
    # Helpers have generic arguments before their parameter list.
    if visibility=='pub':
        rx=re.compile(r'\bpub\s+fn\s+'+re.escape(name)+r'\s*\(')
        matches=list(rx.finditer(code)); sig_start=lambda m:m.start()
    elif visibility is None:
        rx=re.compile(r'\bfn\s+'+re.escape(name)+r'(?=\s*[<(])')
        matches=list(rx.finditer(code)); sig_start=lambda m:m.start()
    else:
        raise AuditError(f'unsupported visibility selector: {visibility}')
    if len(matches)!=1: raise AuditError(f'expected exactly one {visibility+" " if visibility else ""}fn {name}, found {len(matches)}')
    match=matches[0]; fn_start=match.start(); brace=code.find('{',match.end())
    if brace<0: raise AuditError(f'missing body for fn {name}')
    item_start=fn_start
    if visibility is None:
        prefix=re.search(r'\bpub(?:\s*\([^)]*\))?\s*$',code[:fn_start])
        if prefix: item_start=prefix.start()
    start,end,item=item_at(text,code,item_start,f'fn {name}')
    return text[brace+1:end-1],start,end,item,text[sig_start(match):brace]

def fn_data(text: str, name: str, *, visibility: str|None=None)->dict[str,Any]:
    body,start,end,item,sig=find_named_fn(text,name,visibility=visibility)
    body_open=mask_rust_noncode(item).find('{')
    return {'body':body,'start':start,'end':end,'item':item,'signature':sig,'body_start':start+body_open+1,'tokens':tokens(body)}

def attributes_before(text:str,start:int)->list[str]:
    attrs=[]
    raw_lines=text[:start].splitlines(); code_lines=mask_rust_noncode(text[:start]).splitlines()
    for raw,code in zip(reversed(raw_lines),reversed(code_lines)):
        t=raw.strip(); c=code.strip()
        if not c: continue
        if c.startswith('#[') or c.startswith('#!['): attrs.append(t); continue
        break
    return list(reversed(attrs))

def normalize_call_body(body: str)->list[str]:
    return tokens(body)

def source_drop_items(source: str)->dict[str,dict[str,Any]]:
    code=mask_rust_noncode(source)
    all_heads=list(re.finditer(r'\bimpl\s+Drop\s+for\b',code))
    heads=list(re.finditer(r"\bimpl\s+Drop\s+for\s+(SetTrueOnDrop|ToggleOnDrop)\s*<\s*'_\s*>\s*\{",code))
    if len(all_heads)!=2 or len(heads)!=2 or {x.group(1) for x in heads}!={'SetTrueOnDrop','ToggleOnDrop'}:
        raise AuditError('source Drop impl set differs from the two explicit source destructors')
    result={}
    for typ in ('SetTrueOnDrop','ToggleOnDrop'):
        head=next(x for x in heads if x.group(1)==typ)
        start,end,item=item_at(source,code,head.start(),f'impl Drop for {typ}')
        item_code=mask_rust_noncode(item); op=item_code.find('{'); impl_body=item[op+1:-1]
        if len(re.findall(r'\bfn\s+',item_code))!=1: raise AuditError(f'{typ} Drop impl contains extra functions/effects')
        drop_fn=fn_data(item,'drop')
        method_body=drop_fn['body']; fnitem=drop_fn['item']
        result[typ]={'item':item,'body':impl_body,'method':fnitem,'method_body':method_body,'start':start,'end':end,'method_start':start+drop_fn['start'],'tokens':tokens(method_body)}
    return result

def get_scope(source: str, name: str)->dict[str,Any]:
    f=fn_data(source,name,visibility='pub')
    sig=compact(f['signature'])
    if tokens(sig)!=['pub','fn',name,'(','flag',':','&','mut','bool',')']:
        raise AuditError(f'{name} signature differs from the scope subset')
    return f

def expected_source_body(scope: str)->list[str]:
    return {
        'set_true_scope':['let','_guard','=','SetTrueOnDrop','(','flag',')',';'],
        'toggle_scope':['let','_guard','=','ToggleOnDrop','(','flag',')',';'],
        'toggle_after_write':['let','guard','=','ToggleOnDrop','(','flag',')',';','*','guard','.','0','=','true',';'],
    }[scope]

def check_source_profile(source: str, shadow: str)->tuple[dict[str,dict[str,Any]],dict[str,dict[str,Any]],dict[str,dict[str,Any]]]:
    # Known native and shadow layouts: the only fields are borrowed bool references.
    for typ in ('SetTrueOnDrop','ToggleOnDrop'):
        pat=rf"pub struct {typ}<'a>\s*\(pub &'a mut bool\);"
        if len(re.findall(pat,source))!=1 or len(re.findall(pat,shadow))!=1:
            raise AuditError(f'unsupported/drop-glue field layout for {typ}')
    dtypes=source_drop_items(source)
    for typ,drop in dtypes.items():
        if attributes_before(source,drop['start']) or attributes_before(source,drop['method_start']):
            raise AuditError(f'{typ} native Drop implementation has unsupported/trusted attributes')
    src_set=fn_data(source,'set_true'); sh_set=fn_data(shadow,'set_true')
    if src_set['tokens']!=['*','flag','=','true',';'] or sh_set['tokens']!=src_set['tokens']:
        raise AuditError('set_true body differs from its actual simple source operation or is not preserved in the shadow')
    set_sig=['fn','set_true','(','flag',':','&','mut','bool',')']
    if (tokens(src_set['signature'])!=set_sig or tokens(sh_set['signature'])!=set_sig or
        tokens(src_set['item'][:mask_rust_noncode(src_set['item']).find('{')])!=set_sig or
        tokens(sh_set['item'][:mask_rust_noncode(sh_set['item']).find('{')])!=set_sig):
        raise AuditError('set_true signature differs between native and proof shadow')
    exact_attrs={'set_true':['#[ensures(^flag == true)]']}
    if attributes_before(source,src_set['start'])!=exact_attrs['set_true'] or attributes_before(shadow,sh_set['start'])!=exact_attrs['set_true']:
        raise AuditError('set_true source/shadow attributes differ or include trust/unsupported attributes')
    # The shadow has a deliberately tiny annotation vocabulary. This rejects
    # cfg_attr, trusted, axioms and any unsupported attribute anywhere in it.
    allowed_attrs={
      '#[ensures(^flag == true)]','#[requires(*flag == false)]','#[ensures(^flag == !*flag)]','#[ensures(^flag == false)]',
      '#[ensures(^(guard.0) == ^((^guard).0))]','#[ensures(*(^guard).0 == true)]','#[ensures(*(^guard).0 == !*guard.0)]',
      '#[ensures(*(^guard).0 == false)]','#[ensures(*(^guard).0 == *guard.0)]'
    }
    attrs=[line.strip() for line in shadow.splitlines() if line.strip().startswith('#[') or line.strip().startswith('#![')]
    if any(a not in allowed_attrs for a in attrs): raise AuditError(f'unsupported/trusted/axiom/cfg_attr annotation in proof shadow: {attrs!r}')
    if any('#[' in line and not line.strip().startswith('#[') for line in shadow.splitlines()): raise AuditError('noncanonical or hidden shadow attribute')
    if re.search(r'\bimpl\s+Drop\s+for\b',re.sub(r'//[^\n]*|/\*.*?\*/','',shadow,flags=re.S)):
        raise AuditError('proof shadow retains an impl Drop (would double execute the destructor)')
    if re.search(r'\b(?:unsafe|panic|spawn|thread|catch_unwind|forget|ManuallyDrop|Box|RawPtr|NonNull|trusted|axiom|assume|cfg_attr)\b',shadow):
        raise AuditError('unsupported unsafe/panic/thread/escape/raw-pointer construct in shadow')
    scopes={}; shadows={}
    required_scope_attrs={'set_true_scope':['#[requires(*flag == false)]','#[ensures(^flag == true)]'],'toggle_scope':['#[ensures(^flag == !*flag)]'],'toggle_after_write':['#[ensures(^flag == false)]']}
    for spec in EFFECTS:
        src=get_scope(source,spec['scope']); sh=get_scope(shadow,spec['scope'])
        if attributes_before(source,src['start'])!=required_scope_attrs[spec['scope']] or attributes_before(shadow,sh['start'])!=required_scope_attrs[spec['scope']]:
            raise AuditError(f'{spec["scope"]} source/shadow specification attributes changed or contain trust/unsupported annotations')
        if src['tokens']!=expected_source_body(spec['scope']):
            raise AuditError(f'unsupported source events in {spec["scope"]}: {src["tokens"]!r}')
        scopes[spec['scope']]=src; shadows[spec['scope']]=sh
    # Only a known direct helper function is allowed in copied destructor bodies.
    dset=dtypes['SetTrueOnDrop']['tokens']; dtoggle=dtypes['ToggleOnDrop']['tokens']
    allowed_drop_bodies={
      ('set_true','(','self','.','0',')',';'),
      ('*','self','.','0','=','!','*','self','.','0',';'),
      ('*','self','.','0','=','false',';'), # synchronized body mutation reaches a body VC; checker does not prove it
    }
    if tuple(dset) not in allowed_drop_bodies or tuple(dtoggle) not in allowed_drop_bodies:
        # Unknown calls are explicitly distinguished from a failed semantic VC.
        raise AuditError('unknown callback or unsupported destructor body outside the narrow copyable subset')
    if len(re.findall(r'pub\s+fn\s+(?:set_true_scope|toggle_scope|toggle_after_write)\s*\(',source))!=3:
        raise AuditError('native scope set changed')
    if re.search(r'\b(?:unsafe|panic|spawn|thread|catch_unwind|forget|ManuallyDrop)\b',source):
        # Ignore test assertions, but no forbidden construct is needed by these native tests either.
        raise AuditError('unsupported unsafe/panic/thread/escape source construct')
    return dtypes,scopes,shadows

def extract_mir_fn(text: str, scope: str)->str:
    marker=f'fn {scope}(_1: &mut bool) -> () {{'
    start=text.find(marker)
    if start<0: raise AuditError(f'pinned MIR function header missing: {scope}')
    _,end,item=item_after(text,marker[:-2])
    return item

def parse_mir(scope: str, text: str, typ: str, binding: str)->dict[str,Any]:
    if f'after ElaborateDrops' not in text.splitlines()[0]: raise AuditError(f'{scope} MIR is not post-ElaborateDrops')
    body=extract_mir_fn(text,scope)
    if re.search(r'\b(switchInt|assert|yield|coroutineDrop|unwind\s+terminate)\b|goto\s*->',body):
        raise AuditError(f'unsupported control-flow/MIR effect in {scope}')
    drops=list(re.finditer(r'\bdrop\(([^)]*)\)\s*->\s*\[return:\s*(bb\d+),\s*unwind:\s*(bb\d+)\];',body))
    if len(drops)!=1: raise AuditError(f'{scope} needs exactly one native Drop terminator, found {len(drops)}')
    d=drops[0]; place=d.group(1).strip(); normal=d.group(2); unwind=d.group(3)
    if place!='_2': raise AuditError(f'{scope} Drop target differs from local _2: {place}')
    types=re.findall(r'^\s*let(?:\s+mut)?\s+_2:\s*([^;]+);',body,re.M)
    if len(types)!=1 or compact(types[0])!=f"{typ}<'_>": raise AuditError(f'{scope} local _2 type differs: {types!r}')
    debug=re.findall(r'\bdebug\s+([A-Za-z_][A-Za-z0-9_]*)\s*=>\s*_2\s*;',body)
    if debug!=[binding]: raise AuditError(f'{scope} MIR debug local _2 maps to {debug!r}, expected {binding}')
    blocks=list(re.finditer(r'^\s*(bb\d+)(?:\s*\(cleanup\))?:\s*\{',body,re.M))
    names=[x.group(1) for x in blocks]
    if names!=['bb0','bb1','bb2']: raise AuditError(f'{scope} has unsupported basic-block topology {names!r}')
    def block(i:int)->str:
        begin=blocks[i].end(); end=blocks[i+1].start() if i+1<len(blocks) else body.rfind('}')
        return body[begin:end]
    bb0,bb1,bb2=block(0),block(1),block(2)
    if (normal,unwind)!=('bb1','bb2'): raise AuditError(f'{scope} normal/unwind targets changed: {normal}/{unwind}')
    if compact(bb1)!='StorageDead(_2); return; }': raise AuditError(f'{scope} normal successor block contains extra/unrecognized statements')
    block2_header=body[blocks[2].start():blocks[2].end()]
    if '(cleanup)' not in block2_header or compact(bb2)!='resume; }':
        raise AuditError(f'{scope} unwind successor is not the exact cleanup/resume block')
    if re.search(r'\b(drop|call)\s*\(',body.replace(d.group(0),'')):
        raise AuditError(f'{scope} has secondary field-drop/call effects')
    if re.search(r'\bmove\s+_2\b|\bdrop\(_2\.',body): raise AuditError(f'{scope} moves/drops the guard or has unsupported field drop glue')
    stmtlines=[compact(x) for x in bb0.splitlines() if x.strip() and x.strip()!='}']
    di=next(i for i,x in enumerate(stmtlines) if x==compact(d.group(0)))
    expected_prefix={
      'set_true_scope':['StorageLive(_2);','StorageLive(_3);',"_3 = &'_ mut (*_1);", "_2 = SetTrueOnDrop::<'_>(move _3);",'StorageDead(_3);','_0 = const ();'],
      'toggle_scope':['StorageLive(_2);','StorageLive(_3);',"_3 = &'_ mut (*_1);", "_2 = ToggleOnDrop::<'_>(move _3);",'StorageDead(_3);','_0 = const ();'],
      'toggle_after_write':['StorageLive(_2);','StorageLive(_3);',"_3 = &'_ mut (*_1);", "_2 = ToggleOnDrop::<'_>(move _3);",'StorageDead(_3);','_4 = deref_copy (_2.0: &mut bool);','(*_4) = const true;','_0 = const ();'],
    }[scope]
    if stmtlines[:di]!=[compact(x) for x in expected_prefix]:
        raise AuditError(f'{scope} source/MIR statement sequence differs: {stmtlines[:di]!r}')
    if stmtlines[di+1:]: raise AuditError(f'{scope} has statements after the Drop terminator in its normal block')
    if re.search(r'\b(?:call|assert|switchInt|yield)\b',body): raise AuditError(f'{scope} MIR contains unknown call/control flow')
    return {'body':body,'body_sha256':sha(body.encode()),'drop':{'basic_block':'bb0','place':place,'type':f"{typ}<'_>",'terminator':d.group(0),'normal_target':normal,'unwind_target':unwind,'order_in_scope':0},'source_binding':binding,'normal_block':compact(bb1),'unwind_block':compact(bb2),'bb0_statements':stmtlines,'drop_index':di}

def contract_lines(text: str, start: int)->list[str]:
    return attributes_before(text,start)

def call_occurrences(body: str, helper: str, body_start:int=0)->list[dict[str,Any]]:
    # This scope uses only direct helper calls with exactly one &mut local argument.
    rx=re.compile(r'\b'+re.escape(helper)+r'\s*\(\s*&\s*mut\s+([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*;')
    out=[]
    code=mask_rust_noncode(body)
    for m in rx.finditer(code):
        gs=body_start+m.start(); ge=body_start+m.end()
        raw_text=body[m.start():m.end()]
        out.append({'argument':m.group(1),'text':raw_text,'start':m.start(),'end':m.end(),'byte_span':[gs,ge],'line':None,'column':None,'text_sha256':sha(raw_text.encode())})
    return out

def remove_calls(body: str, calls:list[dict[str,Any]])->str:
    out=body
    for c in sorted(calls,key=lambda x:x['start'],reverse=True): out=out[:c['start']]+out[c['end']:]
    return out

def scope_shadow_body(shadow:str, scope:str)->str:
    return fn_data(shadow,scope,visibility='pub')['body']

def check_mapping_receipt(root:pathlib.Path, mapping:dict[str,Any], *, source:str, shadow:str, shadow_rel:str, mapping_rel:str, mir_data:dict[str,str], parsed:dict[str,dict[str,Any]], source_info:dict[str,Any], shadow_info:dict[str,Any], calls_info:dict[str,Any], features:set[str])->None:
    if mapping.get('schema_version')!=1: raise AuditError('unsupported mapping schema')
    if mapping.get('variant',{}).get('features',[])!=sorted(features): raise AuditError('mapping feature selection differs from CLI')
    if mapping.get('variant',{}).get('shadow_sha256')!=sha(shadow.encode()): raise AuditError('shadow receipt hash mismatch')
    if mapping.get('variant',{}).get('mapping_path')!=mapping_rel: raise AuditError('mapping selected path differs from invocation')
    canonical_shadow=root/mapping.get('variant',{}).get('shadow_path','')
    if not canonical_shadow.is_file() or sha(canonical_shadow.read_bytes())!=sha(shadow.encode()): raise AuditError('invoked shadow is not the byte-identical source named by the mapping')
    generator=mapping.get('generator',{})
    if generator.get('sha256')!=sha((root/'elaborate.py').read_bytes()): raise AuditError('mapping generator receipt does not match current elaborator source')
    tool=mapping.get('toolchain',{})
    if tool.get('rustc_version')!=PIN_RELEASE or tool.get('rustc_commit')!=PIN_COMMIT or tool.get('mir_stage')!=PIN_STAGE: raise AuditError('mapping toolchain/stage receipt differs from the pin')
    for flag in ('-Zdump-mir=all','-Zmir-opt-level=0','-Zidentify-regions=yes','-Copt-level=0','-Cpanic=unwind'):
        if flag not in tool.get('rustc_flags',''): raise AuditError(f'mapping compiler flags omit {flag}')
    effects=mapping.get('effects')
    if not isinstance(effects,list) or len(effects)!=3: raise AuditError('mapping effect count does not equal independently parsed MIR Drop count')
    if sha(source.encode())!=mapping.get('inputs',{}).get('source_sha256'): raise AuditError('native.rs receipt hash mismatch')
    for i,spec in enumerate(EFFECTS):
        e=effects[i]
        if (e.get('order'),e.get('scope'))!=(i,spec['scope']): raise AuditError(f'mapping effect ordering/name mismatch at {i}')
        if e['source_scope'].get('source_binding')!=parsed[spec['scope']]['source_binding']: raise AuditError(f'{spec["scope"]} source binding receipt mismatch')
        src=source_info[spec['scope']]
        if e['source_scope'].get('signature')!=f"pub fn {spec['scope']}" or e['source_scope'].get('body')!=src['body'] or e['source_scope'].get('body_sha256')!=sha(src['body'].encode()): raise AuditError(f'{spec["scope"]} source-scope receipt differs from reparse')
        if e['source_scope'].get('start_line')!=source[:src['start']].count('\n')+1: raise AuditError(f'{spec["scope"]} source-scope line receipt mismatch')
        m=e['native_mir']; rawmir=mir_data[spec['scope']]
        rel='native-mir/'+spec['mir']
        if m.get('path')!=rel or m.get('sha256')!=sha(rawmir.encode()): raise AuditError(f'{spec["scope"]} MIR receipt path/hash mismatch')
        if m.get('body')!=parsed[spec['scope']]['body'] or m.get('body_sha256')!=parsed[spec['scope']]['body_sha256']: raise AuditError(f'{spec["scope"]} MIR body receipt mismatch')
        expected_drop=parsed[spec['scope']]['drop']
        if m.get('drop')!={**expected_drop,'unwind_interpretation':'recorded cleanup edge; not elaborated into a proof effect'}: raise AuditError(f'{spec["scope"]} Drop-edge receipt differs from MIR parse')
        if m.get('debug_local_source_name')!=spec['binding']: raise AuditError(f'{spec["scope"]} source-local receipt mismatch')
        ds=e['source_destructor']; srcdrop=source_drop_items(source)[spec['type']]
        if ds.get('type')!=spec['type'] or ds.get('body')!=srcdrop['body'] or ds.get('body_sha256')!=sha(srcdrop['body'].encode()) or compact(ds.get('method_body',''))!=compact(srcdrop['method']): raise AuditError(f'{spec["scope"]} source destructor receipt differs from native source')
        if ds.get('method_body_sha256')!=sha(ds['method_body'].encode()) or ds.get('start_line')!=source[:srcdrop['start']].count('\n')+1: raise AuditError(f'{spec["scope"]} destructor source hash/line receipt mismatch')
        method_at=source.find('fn drop',srcdrop['start'])
        if ds.get('method_start_byte')!=method_at: raise AuditError(f'{spec["scope"]} destructor byte-offset receipt mismatch')
        h=e['shadow_helper']; actual=shadow_info[spec['helper']]
        if h.get('name')!=spec['helper'] or h.get('path')!=mapping.get('variant',{}).get('shadow_path'): raise AuditError(f'{spec["scope"]} helper receipt identity mismatch')
        for field,observed in [('body',actual['body']),('body_sha256',sha(actual['body'].encode())),('item_sha256',sha(actual['item'].encode()))]:
            if h.get(field)!=observed: raise AuditError(f'{spec["scope"]} helper {field} receipt mismatch')
        hs=shadow[:actual['start']].count('\n')+1; he=shadow[:actual['end']].count('\n')+1
        if h.get('span_bytes')!=[actual['start'],actual['end']] or h.get('span_lines')!=[hs,he]: raise AuditError(f'{spec["scope"]} helper source span receipt mismatch')
        if compact(h.get('copied_from_drop_body',''))!=compact(ds.get('method_body','')): raise AuditError(f'{spec["scope"]} copied-destructor receipt mismatch')
        contracts=h.get('contracts',{})
        if contracts.get('frame')!='#[ensures(^(guard.0) == ^((^guard).0))]': raise AuditError(f'{spec["scope"]} frame contract receipt mismatch')
        actual_attrs=contract_lines(shadow,actual['start'])
        active_value=next((x for x in actual_attrs if x.startswith('#[ensures(*(^guard).0 ==')),None)
        if contracts.get('value')!=active_value: raise AuditError(f'{spec["scope"]} active value-contract receipt mismatch')
        if contracts.get('positive_value') and contracts.get('positive_value') not in ('#[ensures(*(^guard).0 == true)]','#[ensures(*(^guard).0 == !*guard.0)]'): raise AuditError(f'{spec["scope"]} positive value-contract receipt is not a checked body summary')
        inj=e['injection']; expected_call=f"{spec['helper']}(&mut {spec['binding']});"
        if inj.get('expected_call')!=expected_call: raise AuditError(f'{spec["scope"]} expected-call receipt mismatch')
        if inj.get('normal_edge')!='bb1' or inj.get('unwind_edge')!='bb2' or inj.get('mode')!='normal-return only; unwind edge recorded and not claimed': raise AuditError(f'{spec["scope"]} normal/unwind scope receipt mismatch')
        if inj.get('expected_call_count')!=len(calls_info[spec['scope']]): raise AuditError(f'{spec["scope"]} call-count receipt mismatch')
        recorded=inj.get('observed_generated_calls',[])
        actualcalls=calls_info[spec['scope']]
        if len(recorded)!=len(actualcalls): raise AuditError(f'{spec["scope"]} generated call-site receipt count mismatch')
        shbody=scope_shadow_body(shadow, spec['scope'])
        if inj.get('generated_function_body_sha256')!=sha(shbody.encode()): raise AuditError(f'{spec["scope"]} generated scope body hash receipt mismatch')
        # Validate the receipt's raw call text/argument without using its offsets as evidence.
        for row,call in zip(recorded,actualcalls):
            g=call['byte_span'][0]; line=shadow[:g].count('\n')+1; last=shadow.rfind('\n',0,g); col=g-last
            if row.get('text')!=call['text'] or row.get('argument')!=call['argument'] or row.get('order_in_scope')!=actualcalls.index(call) or row.get('byte_span')!=call['byte_span'] or row.get('line')!=line or row.get('column')!=col or row.get('text_sha256')!=call['text_sha256']: raise AuditError(f'{spec["scope"]} call-site receipt differs from parsed shadow')
    if mapping.get('normal_return_scope_only') is not True or mapping.get('unwind_edges_are_recorded_not_claimed') is not True: raise AuditError('mapping overclaims normal/unwind scope')

def run_check(root:pathlib.Path, shadow_path:pathlib.Path, mapping_path:pathlib.Path, features:set[str], *, verify_receipt:bool=True)->dict[str,Any]:
    source_path=root/'native.rs'; source=source_path.read_text()
    shadow=shadow_path.read_text(); mapping=json.loads(mapping_path.read_text())
    dtypes,source_info,scope_shadow=check_source_profile(source,shadow)
    # Verify every shadow scope is the original executable source body plus one
    # trailing normal-exit call. `mut` is required only to borrow the local into
    # the proof helper; it is erased for source-event comparison.
    helpers={e['scope']:e['helper'] for e in EFFECTS}
    parsed={}; mir_data={}; calls_by_scope={}; shadow_info={}
    for typ, spec in [('SetTrueOnDrop',EFFECTS[0]),('ToggleOnDrop',EFFECTS[1])]:
        # method body extraction above records Drop impl body; compare below to helper.
        pass
    for spec in EFFECTS:
        sh=scope_shadow[spec['scope']]
        expected_tokens=source_info[spec['scope']]['tokens']
        calls=call_occurrences(sh['body'],spec['helper'],sh['body_start'])
        for c in calls:
            c['line']=shadow[:c['byte_span'][0]].count('\n')+1; last=shadow.rfind('\n',0,c['byte_span'][0]); c['column']=c['byte_span'][0]-last
        wrong_helpers=[]
        for other in EFFECTS:
            if other['helper']!=spec['helper']:
                wrong_helpers.extend(call_occurrences(sh['body'],other['helper'],sh['body_start']))
        if wrong_helpers: raise AuditError(f'{spec["scope"]} calls the wrong destructor helper')
        if len(calls)!=1: raise AuditError(f'{spec["scope"]} needs exactly one helper call for its one normal Drop edge; saw {len(calls)}')
        call=calls[0]
        if call['argument']!=spec['binding']: raise AuditError(f'{spec["scope"]} helper targets {call["argument"]}, not MIR Drop local `{spec["binding"]}`')
        # Ensure this is the final event before normal return, and preserve every
        # original source token outside the single call and let mut adjustment.
        if sh['body'][call['end']:].strip(): raise AuditError(f'{spec["scope"]} helper call is not on the final normal-return edge')
        rest=tokens(remove_calls(sh['body'],calls))
        rest=[x for i,x in enumerate(rest) if not (x=='mut' and i>0 and rest[i-1]=='let')]
        if rest!=expected_tokens: raise AuditError(f'{spec["scope"]} source events are not preserved in proof shadow')
        if spec['scope']=='toggle_after_write':
            assignment='*guard.0 = true;'
            assignment_pos=sh['body'].find(assignment)
            if assignment_pos<0 or assignment_pos>=call['start']: raise AuditError('toggle_after_write Drop effect is not after the native write')
        calls_by_scope[spec['scope']]=calls
    # Parse destructor helpers independently and compare copied token bodies.
    drop_src=source_drop_items(source)
    for typ, helper in [('SetTrueOnDrop','set_true_drop_effect'),('ToggleOnDrop','toggle_drop_effect')]:
        hf=fn_data(shadow,helper)
        shadow_info[helper]=hf
        source_tokens=drop_src[typ]['tokens']
        copied=['guard' if x=='self' else x for x in source_tokens]
        if hf['tokens']!=copied: raise AuditError(f'{helper} body is not the source Drop body modulo receiver-to-parameter rename')
        if re.search(r'\bunsafe\b|\bpanic\b|\bspawn\b|\bthread\b|\bforget\b',hf['body']): raise AuditError(f'{helper} has unsupported effect')
        sig=compact(hf['signature'])
        expected_sig=f"fn {helper}<'a>(guard: &mut {typ}<'a>)"
        header=mask_rust_noncode(hf['item']).split('{',1)[0]
        if sig!=expected_sig or tokens(header)!=['pub','(','crate',')']+tokens(expected_sig):
            raise AuditError(f'{helper} visibility/type/parameter differs from the required proof-only adapter: {sig}')
        # Frame postcondition is required; wrong value summaries are structural
        # correspondence-preserving and are reported for the helper body VC.
        attrs=contract_lines(shadow,hf['start'])
        frame='#[ensures(^(guard.0) == ^((^guard).0))]'
        positive_value=('#[ensures(*(^guard).0 == true)]' if typ=='SetTrueOnDrop' else '#[ensures(*(^guard).0 == !*guard.0)]')
        wrong_value=('#[ensures(*(^guard).0 == false)]' if typ=='SetTrueOnDrop' else '#[ensures(*(^guard).0 == *guard.0)]')
        expected_attrs=[frame, wrong_value] if 'wrong_drop_summary' in features else [frame, positive_value]
        if attrs!=expected_attrs: raise AuditError(f'{helper} attributes differ from the exact allowed frame/value pair or include trust/axioms')
    # Validate scope specifications are copied exactly (these public bodies are
    # not used as TCB assumptions).
    for name in ('set_true_scope','toggle_scope','toggle_after_write'):
        attrs_src=contract_lines(source,source_info[name]['start']); attrs_sh=contract_lines(shadow,scope_shadow[name]['start'])
        if attrs_src!=attrs_sh: raise AuditError(f'{name} proof contract changed in generated shadow')
    # Source-elided compiler input and pinned MIR provenance.
    native_input=(root/'native-mir/native-rustc-input.rs').read_text()
    allowed={'use creusot_std::prelude::*;','#[ensures(^flag == true)]','#[requires(*flag == false)]','#[ensures(^flag == !*flag)]','#[ensures(^flag == false)]'}
    kept=[]; removed=[]
    for line_no,line in enumerate(source.splitlines(keepends=True),1):
        if line.strip() in allowed:
            removed.append({'line':line_no,'text':line.strip()}); kept.append('\n' if line.endswith('\n') else '')
        else: kept.append(line)
    if ''.join(kept)!=native_input: raise AuditError('native rustc input is not source with only exact verifier-only syntax erased')
    erasure=json.loads((root/'native-mir/source-erasure.json').read_text())
    if erasure.get('source_sha256')!=sha(source.encode()) or erasure.get('rustc_input_sha256')!=sha(native_input.encode()) or erasure.get('erased_spec_syntax')!=removed or erasure.get('executable_source_preserved') is not True:
        raise AuditError('source-erasure receipt does not match independently reconstructed input')
    version=(root/'native-mir/rustc-version.txt').read_text()
    command=(root/'native-mir/command.txt').read_text()
    for required in (PIN_RELEASE,PIN_COMMIT):
        if required not in version or required not in command: raise AuditError(f'pinned rustc provenance missing {required}')
    for flag in ('-Zdump-mir=all','-Zmir-opt-level=0','-Zidentify-regions=yes','-Copt-level=0','-Cpanic=unwind',PIN_STAGE):
        if flag not in command: raise AuditError(f'pinned MIR command missing {flag}')
    for spec in EFFECTS:
        rel=root/'native-mir'/spec['mir']; text=rel.read_text(); mir_data[spec['scope']]=text
        parsed[spec['scope']]=parse_mir(spec['scope'],text,spec['type'],spec['binding'])
    if verify_receipt:
        try:
            shadow_rel=shadow_path.resolve().relative_to(root.resolve()).as_posix(); mapping_rel=mapping_path.resolve().relative_to(root.resolve()).as_posix()
        except ValueError: raise AuditError('shadow and mapping must resolve within the probe root')
        check_mapping_receipt(root,mapping,source=source,shadow=shadow,shadow_rel=shadow_rel,mapping_rel=mapping_rel,mir_data=mir_data,parsed=parsed,source_info=source_info,shadow_info=shadow_info,calls_info=calls_by_scope,features=features)
    semantic=[]
    for typ,helper in [('SetTrueOnDrop','set_true_drop_effect'),('ToggleOnDrop','toggle_drop_effect')]:
        attrs=contract_lines(shadow,shadow_info[helper]['start'])
        expected_value='#[ensures(*(^guard).0 == true)]' if typ=='SetTrueOnDrop' else '#[ensures(*(^guard).0 == !*guard.0)]'
        if expected_value not in attrs: semantic.append({'helper':helper,'classification':'semantic_vc_required','reason':'body/copy correspondence alone cannot validate a weakened or wrong effect contract; proof body must reject it'})
    result={
      'schema_version':1,'status':'correspondence_pass','features':sorted(features),
      'checker':'check_correspondence.py; parses native source, rustc input, raw pinned MIR, proof shadow and mapping receipt independently',
      'source_sha256':sha(source.encode()),'shadow_sha256':sha(shadow.encode()),
      'compiler':{'release':PIN_RELEASE,'commit':PIN_COMMIT,'mir_stage':PIN_STAGE,'optimization_level':0,'panic':'unwind'},
      'effects':[],
      'semantic_vc_controls':semantic,
      'scope':'three closed straight-line normal-return scope exits; cleanup/unwind edges observed and deliberately not modeled as proof effects',
      'trust_boundary':'The checker and pinned MIR-to-shadow injection are generic tool TCB. This audit does not establish checker/compiler soundness, arbitrary Rust Drop semantics, destructor panic/unwind effects, field drop glue, or any Bytes ownership/refcount law.'
    }
    for spec in EFFECTS:
        p=parsed[spec['scope']]
        calls=calls_by_scope[spec['scope']]
        result['effects'].append({'scope':spec['scope'],'source_type':spec['type'],'mir_place':p['drop']['place'],'mir_type':p['drop']['type'],'mir_block':p['drop']['basic_block'],'normal_target':p['drop']['normal_target'],'unwind_target':p['drop']['unwind_target'],'normal_successor':p['normal_block'],'unwind_successor':p['unwind_block'],'helper':spec['helper'],'helper_argument':calls[0]['argument'],'helper_call_after_source_events':True,'mir_sha256':sha(mir_data[spec['scope']].encode()),'mir_body_sha256':p['body_sha256']})
    return result

def main()->int:
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--shadow',type=pathlib.Path,default=ROOT/'generated/shadow.rs')
    ap.add_argument('--mapping',type=pathlib.Path,default=ROOT/'generated/mapping.json')
    ap.add_argument('--report',type=pathlib.Path,default=ROOT/'generated/correspondence.json')
    ap.add_argument('--features',default='',help='one diagnostic feature label; validates mapping variant')
    args=ap.parse_args()
    features={x for x in args.features.split(',') if x}
    if len(features)>1 or features-FEATURES:
        print(json.dumps({'status':'coverage_failure','error':'only one known diagnostic feature is supported'}),file=sys.stderr); return 2
    try:
        result=run_check(ROOT,args.shadow,args.mapping,features)
        args.report.parent.mkdir(parents=True,exist_ok=True)
        args.report.write_text(json.dumps(result,indent=2)+'\n')
        print(json.dumps({'status':result['status'],'report':str(args.report),'effects':len(result['effects']),'semantic_vc_controls':result['semantic_vc_controls']}))
        return 0
    except (OSError,ValueError,KeyError,IndexError,AuditError) as e:
        category=e.category if isinstance(e,AuditError) else 'coverage_failure'
        failed={'schema_version':1,'status':category,'features':sorted(features),'error':str(e),'trust_boundary':'No proof result is implied by correspondence acceptance.'}
        try:
            args.report.parent.mkdir(parents=True,exist_ok=True); args.report.write_text(json.dumps(failed,indent=2)+'\n')
        except OSError: pass
        print(json.dumps(failed),file=sys.stderr)
        return 2
if __name__=='__main__': raise SystemExit(main())
