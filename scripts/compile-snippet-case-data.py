#!/usr/bin/env python3
"""Print frozen source data; never rewrite an owning page or generated output."""
import argparse,json,subprocess,tempfile,unicodedata
from pathlib import Path
parser=argparse.ArgumentParser();parser.add_argument("unicode_tables",type=Path,help="regex-syntax 0.8.11 src/unicode_tables directory");args=parser.parse_args()
if unicodedata.unidata_version!="16.0.0":raise SystemExit("Use a Python interpreter with Unicode 16.0.0 tables (Python 3.14)")
base=args.unicode_tables.resolve();source="#![allow(dead_code)]\n"
for name,file in [("property","property_bool"),("category","general_category"),("fold","case_folding_simple")]:
 path=base/(file+".rs")
 if "Unicode version: 16.0.0." not in path.read_text():raise SystemExit("Unexpected Unicode table version")
 source+=f'#[path={json.dumps(str(path))}] mod {name};\n'
source+=r"""
fn main(){fn ranges(v:&[(char,char)])->Vec<[u32;2]>{v.iter().map(|(a,b)|[*a as u32,*b as u32]).collect()}
println!("{{\"letter\":{:?},\"cased\":{:?},\"ignorable\":{:?},\"identifier\":{:?},\"fold\":{:?}}}",ranges(category::LETTER),ranges(property::CASED),ranges(property::CASE_IGNORABLE),ranges(property::XID_CONTINUE),fold::CASE_FOLDING_SIMPLE.iter().map(|(c,items)|(*c as u32,items.iter().map(|c|*c as u32).collect::<Vec<_>>())).collect::<Vec<_>>());}
"""
# Rust debug tuples are converted to JSON arrays only in this numeric exporter.
with tempfile.TemporaryDirectory(prefix="hickory-snippet-case-") as temporary:
 root=Path(temporary);(root/"properties.rs").write_text(source);subprocess.run(["rustc",str(root/"properties.rs"),"-o",str(root/"properties")],check=True);raw=subprocess.check_output([str(root/"properties")],text=True).replace("(","[").replace(")","]");data=json.loads(raw)
data={"unicode":"16.0.0",**data,"version":"hickory-ucd-16.0.0-v1"}
for name,convert in [("upper",str.upper),("lower",str.lower)]:data[name]=[[cp,convert(chr(cp))] for cp in range(0x110000) if not 0xD800<=cp<=0xDFFF and convert(chr(cp))!=chr(cp)]
print(json.dumps(data,ensure_ascii=False,separators=(",",":")))
