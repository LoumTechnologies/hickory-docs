// Guarantee: editing/a-comparison-keeps-current-code-editable.md
import { expect, it } from "vitest";
import { filePosition, representationOffset, representationRange, type RepresentedFile } from "./representationMapping";
const bytes=(s:string)=>new TextEncoder().encode(s).length;
it("maps reordered literate fragments to real files, including Unicode and boundaries",()=>{
  const first="é = 1\n", second="other = 2\n";
  const source=`# Explain\n${second}# Earlier\n${first}`;
  const a=source.indexOf(first), b=source.indexOf(second);
  const file:RepresentedFile={path:"a.py",hash:"",content:first+second,provenance:[
    {start:0,end:bytes(first),origin:{kind:"paste",doc_path:"virtual",span:[bytes(source.slice(0,a)),bytes(source.slice(0,a+first.length))]}},
    {start:bytes(first),end:bytes(first+second),origin:{kind:"paste",doc_path:"virtual",span:[bytes(source.slice(0,b)),bytes(source.slice(0,b+second.length))]}}
  ]};
  expect(filePosition(source,file,a+2)).toEqual({line:0,character:2});
  expect(filePosition(source,file,b)).toEqual({line:1,character:0});
  expect(representationOffset(source,file,{line:1,character:0})).toBe(b);
  expect(representationOffset(source,file,{line:0,character:2})).toBe(a+2);
  expect(filePosition(source,file,0)).toBeNull();
  expect(filePosition(source,{...file,source_path:"other-document"},a)).toBeNull();
  expect(representationRange(source,file,{start:{line:0,character:0},end:{line:1,character:9}})).toBeNull();
  expect(representationRange(source,file,{start:{line:0,character:0},end:{line:0,character:1}})).toEqual({from:a,to:a+1});
});
it("refuses shared origins and computed coordinates rather than mapping to another occurrence",()=>{
  const file:RepresentedFile={path:"a.py",hash:"",content:"aaa",provenance:[
    {start:0,end:1,origin:{kind:"literal",doc_path:"virtual",span:[0,1]}},
    {start:1,end:2,origin:{kind:"literal",doc_path:"virtual",span:[0,1]}},
    {start:2,end:3,origin:{kind:"synthetic"}}
  ]};
  expect(filePosition("a",file,0)).toBeNull();
  expect(representationOffset("a",file,{line:0,character:2})).toBeNull();
});
