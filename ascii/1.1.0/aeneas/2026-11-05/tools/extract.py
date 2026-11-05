from pathlib import Path
import subprocess,json
r=Path(__file__).resolve().parents[1]
roots = ['ascii::ascii_char::AsciiChar::is_ascii_blank', 'ascii::ascii_char::AsciiChar::is_ascii_digit', 'ascii::ascii_char::AsciiChar::is_ascii_whitespace', 'ascii::ascii_char::AsciiChar::is_ascii_control', 'ascii::ascii_char::AsciiChar::is_ascii_alphabetic', 'ascii::ascii_char::AsciiChar::is_ascii_alphanumeric', 'ascii::ascii_char::AsciiChar::is_ascii_graphic', 'ascii::ascii_char::AsciiChar::is_ascii_printable', 'ascii::ascii_char::AsciiChar::is_ascii_lowercase', 'ascii::ascii_char::AsciiChar::is_ascii_uppercase', 'ascii::ascii_char::AsciiChar::is_ascii_hexdigit', 'ascii::ascii_char::AsciiChar::as_byte', 'ascii::ascii_char::AsciiChar::eq_ignore_ascii_case', '{impl ascii::ascii_char::ToAsciiChar for ascii::ascii_char::AsciiChar}::to_ascii_char', '{impl ascii::ascii_char::ToAsciiChar for ascii::ascii_char::AsciiChar}::to_ascii_char_unchecked', 'ascii::ascii_char::AsciiChar::is_alphabetic', 'ascii::ascii_char::AsciiChar::is_alphanumeric', 'ascii::ascii_char::AsciiChar::is_lowercase', 'ascii::ascii_char::AsciiChar::is_uppercase', 'ascii::ascii_char::AsciiChar::is_whitespace', 'ascii::ascii_char::AsciiChar::is_digit']
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest','evidence']
for root in roots:cmd+=['--start-from',root]
(r/'evidence/extraction-command.json').write_text(json.dumps(cmd,indent=2)+'\n')
with (r/'evidence/charon.log').open('w') as f:subprocess.run(cmd,cwd=r,check=True,stdout=f,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs/Generated','evidence/ascii.llbc']
with (r/'evidence/aeneas.log').open('w') as f:subprocess.run(cmd,cwd=r,check=True,stdout=f,stderr=subprocess.STDOUT)
