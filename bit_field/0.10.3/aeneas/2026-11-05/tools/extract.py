import subprocess,os
args=['charon','cargo','--preset=aeneas','--no-dedup-serialized-ast','--sysroot','default','--start-from','{impl bit_field::BitField for _}::get_bit','--start-from','{impl bit_field::BitField for _}::set_bit','--dest-file','evidence/bit_field.llbc']
for ty in ['u16','u32','u64','u128','usize','i8','i16','i32','i64','i128','isize']:
 args += ['--exclude','{impl bit_field::BitField for '+ty+'}']
subprocess.run(args,check=True,env=dict(os.environ,CARGO_NET_OFFLINE='true'))
subprocess.run(['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest','proofs/BitField','evidence/bit_field.llbc'],check=True)
