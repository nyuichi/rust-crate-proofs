from pathlib import Path
import subprocess,json,re,hashlib
p=Path(__file__).resolve().parents[1];rows=[]
for label,ty in [('Bool','Bool'),('ArrayU32','Aeneas.Std.Array Aeneas.Std.U32 2#usize')]:
 s='import Serializers.Funs\nopen Aeneas Aeneas.Std\n#check XoshiroSerializationModel.core.mem.size_of ('+ty+')\n'
 r=subprocess.run(['lake','env','lean','--stdin'],cwd=p/'proofs',input=s,text=True,capture_output=True)
 (p/('evidence/serializer-negative-'+label+'.log')).write_text(r.stdout+r.stderr)
 assert r.returncode!=0 and 'failed to synthesize' in r.stdout
 rows.append({'type':ty,'rejected':True,'exit':r.returncode})
(p/'evidence/serializer-negative.json').write_text(json.dumps(rows,indent=2))
constants=re.findall(r'^def ([\w.]+state\.N) :', (p/'proofs/Serializers/Funs.lean').read_text(),re.M)
assert len(constants)==11
s='import SerializerProofs\nimport SerializerLoopProofs\nset_option pp.explicit true\n'+'\n'.join('#print XoshiroSerializationModel.'+c for c in constants)+'\n'
r=subprocess.run(['lake','env','lean','--stdin'],cwd=p/'proofs',input=s,text=True,capture_output=True,check=True)
assert r.stdout.count('XoshiroSerializationModel.layout_u32')==5 and r.stdout.count('XoshiroSerializationModel.layout_u64')==6
(p/'evidence/serializer-frozen-layout.log').write_text(r.stdout+r.stderr)
print('Bool/Array unknown evidence rejected; generated constants freeze exactly U32/U64 instance arguments')

provider=p/'proofs/Serializers/FunsExternal.lean'
assert re.findall(r'^instance (\w+) :',provider.read_text(),re.M)==['layout_u32','layout_u64']
(p/'evidence/serializer-model-interface.json').write_text(json.dumps({'open_evidence_class':True,'supplied_instances':['U32=4','U64=8'],'frozen_generated_constants':constants,'provider_sha256':hashlib.sha256(provider.read_bytes()).hexdigest(),'unknown_types_rejected':['Bool','Array U32 2']},indent=2))
