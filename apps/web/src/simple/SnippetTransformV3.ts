import frozenCase from '../../../simple-contract/snippet-regex-case17.json';
export type RegexV3Instruction =
 | ['accept'] | ['jump',number] | ['split',number,number]
 | ['mark'|'capture',number] | ['assert','start'|'end'|'word'|'nonword',boolean,boolean]
 | ['ref',number[],boolean,boolean] | ['set',number[][],number[][],boolean,boolean]
 | ['look',number,boolean,number[]] | ['repeat',number,number,number|null,boolean,number,number,number[]]
 | ['repeat_end',number,number];
export type RegexV3Format = string | ['capture',number,string] | ['if',number,RegexV3Format[],RegexV3Format[]];
export interface RegexV3Program {version:3;case_map:string;unicode_map:string;groups:number;flags:string;code:RegexV3Instruction[];subprograms:{direction:number;code:RegexV3Instruction[]}[];format:RegexV3Format[]}
export interface RegexV3CaseData {fold:[number,number[]][];upper:[number,string][];lower:[number,string][];letter:number[][];cased:number[][];ignorable:number[][];upper_letter:number[][];lower_letter:number[][];space:number[][]}
interface RegexV3Policy {canonical:(cp:number,full:boolean)=>number}
interface RegexV3State {pc:number;at:number;captures:([number,number]|null)[];marks:(number|null)[];repeat:Map<number,{count:number;at:number}>}
interface RegexV3Budget {work:number}

// Pure bounded v3 control-flow evaluator prototype. No eval/native RegExp.
const MAX_WORK=100000,MAX_STACK=1024;
const isScalar=(source:string)=>!/[\uD800-\uDFFF]/u.test(source);
const regexV3PolicyCache=new WeakMap<RegexV3CaseData,RegexV3Policy>();
export function casePolicy(data:RegexV3CaseData):RegexV3Policy{
 const cached=regexV3PolicyCache.get(data);if(cached)return cached;
 const unicode=new Map<number,number>(),legacy=new Map<number,number>();for(const [cp,peers]of data.fold){const least=Math.min(cp,...peers);unicode.set(cp,least);}for(const [cp,text]of data.upper){if(cp>65535||text.length!==1)continue;const value=text.charCodeAt(0);if(cp>=128&&value<128)continue;legacy.set(cp,value);}const policy=Object.freeze({canonical:(cp:number,full:boolean)=> (full?unicode:legacy).get(cp)??cp});regexV3PolicyCache.set(data,policy);return policy;
}
function units(source:string,unicode:boolean):number[]{if(unicode)return Array.from(source,char=>char.codePointAt(0)!);const result=[];for(let at=0;at<source.length;at++)result.push(source.charCodeAt(at));return result;}
function text(points:number[],unicode:boolean):string{let result='';for(let at=0;at<points.length;at+=4096)result+=unicode?String.fromCodePoint(...points.slice(at,at+4096)):String.fromCharCode(...points.slice(at,at+4096));return result;}
export function execute(program:RegexV3Program,input:string,policy:RegexV3Policy,budget:RegexV3Budget={work:0}){
 if(input.length>65536||!isScalar(input)||new TextEncoder().encode(input).length>65536)throw Error('Regex input budget or scalar validation failed');const unicode=program.flags.includes('u')||program.flags.includes('v'),chars=units(input,unicode),offsets:number[]=[0];for(const cp of chars)offsets.push(offsets[offsets.length-1]+(unicode&&cp>65535?2:1));let retained=0;
 const tick=(cost=1)=>{budget.work+=cost;if(!Number.isSafeInteger(budget.work)||budget.work<0||budget.work>MAX_WORK)throw Error('Regex work budget exceeded');};
 const duplicate=(state:RegexV3State):RegexV3State=>{tick(state.captures.length+state.marks.length+state.repeat.size);return {...state,captures:state.captures.slice(),marks:state.marks.slice(),repeat:new Map(state.repeat)};};
 const captureClear=(state:RegexV3State,indices:number[])=>{for(const index of indices){state.captures[index]=null;state.marks[index]=null;tick();}};
 const newline=(cp:number|undefined)=>cp===10||cp===13||cp===0x2028||cp===0x2029;
 const word=(cp:number|undefined,ignore:boolean)=>{if(cp===undefined)return false;if(ignore)cp=policy.canonical(cp,true);return cp>=65&&cp<=90||cp>=97&&cp<=122||cp>=48&&cp<=57||cp===95;};
 const member=(ranges:number[][],cp:number)=>{let low=0,high=ranges.length;while(low<high){tick();const mid=(low+high)>>>1;if(ranges[mid][0]<=cp)low=mid+1;else high=mid;}return low>0&&cp<=ranges[low-1][1];};
 const run=(code:RegexV3Instruction[],start:number,captures:([number,number]|null)[],marks:(number|null)[],direction:number,depth:number):RegexV3State|null=>{
  if(depth>32)throw Error('Regex assertion nesting budget exceeded');tick(captures.length+marks.length);let state:RegexV3State={pc:0,at:start,captures:captures.slice(),marks:marks.slice(),repeat:new Map()};const stack:RegexV3State[]=[];
  const push=(value:RegexV3State)=>{if(++retained>MAX_STACK)throw Error('Regex backtrack memory budget exceeded');stack.push(value);};
  const fail=()=>{if(!stack.length)return false;state=stack.pop()!;retained--;return true;};
  try{for(;;){tick();const instruction=code[state.pc];if(!instruction)throw Error('Regex instruction outside program');let failed=false;
   switch(instruction[0]){
    case 'accept':return state;
    case 'jump':state.pc=instruction[1];continue;
    case 'split':{const alternative=duplicate(state);alternative.pc=instruction[2];push(alternative);state.pc=instruction[1];continue;}
    case 'mark':state=duplicate(state);state.marks[instruction[1]]=state.at;state.captures[instruction[1]]=null;break;
    case 'capture':{const mark=state.marks[instruction[1]];if(mark===null||mark===undefined)throw Error('Regex capture has no start');state=duplicate(state);state.captures[instruction[1]]=[Math.min(mark,state.at),Math.max(mark,state.at)];break;}
    case 'assert':{const kind=instruction[1],before=chars[state.at-1],next=chars[state.at],boundary=word(before,instruction[3])!==word(next,instruction[3]);const yes=kind==='start'?state.at===0||instruction[2]&&newline(before):kind==='end'?state.at===chars.length||instruction[2]&&newline(next):kind==='word'?boundary:!boundary;failed=!yes;break;}
    case 'ref':{const captured=instruction[1].map(index=>state.captures[index]).filter((value):value is [number,number]=>value!==null);if(captured.length>1)throw Error('Ambiguous named regex reference');if(!captured.length)break;const [from,to]=captured[0],length=to-from,end=state.at+direction*length,begin=Math.min(state.at,end);if(begin<0||end>chars.length){failed=true;break;}for(let at=0;at<length;at++){tick();const expected=chars[from+at],actual=chars[begin+at];if(actual===undefined||(instruction[2]?policy.canonical(expected,instruction[3])!==policy.canonical(actual,instruction[3]):expected!==actual)){failed=true;break;}}if(!failed)state.at=end;break;}
    case 'set':{const candidates=[];for(const value of instruction[2]){tick();const end=state.at+direction*value.length,begin=Math.min(state.at,end);if(begin<0||end>chars.length)continue;let yes=true;for(let at=0;at<value.length;at++){tick();const actual=chars[begin+at];if((instruction[3]?policy.canonical(actual,instruction[4]):actual)!==value[at]){yes=false;break;}}if(yes)candidates.push(value.length);}const next=chars[direction===1?state.at:state.at-1];if(next!==undefined&&member(instruction[1],next))candidates.push(1);candidates.sort((a,b)=>b-a);if(!candidates.length){failed=true;break;}for(let at=candidates.length-1;at>0;at--){const alternative=duplicate(state);alternative.at+=direction*candidates[at];alternative.pc++;push(alternative);}state.at+=direction*candidates[0];break;}
    case 'look':{state=duplicate(state);captureClear(state,instruction[3]);const sub=program.subprograms[instruction[1]],result=run(sub.code,state.at,state.captures,state.marks,sub.direction,depth+1);if(instruction[2]){if(!result){failed=true;break;}state.captures=result.captures;state.marks=result.marks;}else if(result)failed=true;break;}
    case 'repeat':{const [,id,min,max,lazy,body,end,captures]=instruction,frame=state.repeat.get(id)??{count:0,at:state.at};const exit=()=>{const result=duplicate(state);result.repeat.delete(id);result.pc=end;return result;};const more=()=>{const result=duplicate(state);result.repeat.set(id,{count:frame.count,at:state.at});captureClear(result,captures);result.pc=body;return result;};const canExit=frame.count>=min,canRepeat=max===null||frame.count<max;if(canExit&&canRepeat){if(lazy){push(more());state=exit();}else{push(exit());state=more();}}else if(canExit)state=exit();else if(canRepeat)state=more();else failed=true;if(!failed)continue;break;}
    case 'repeat_end':{const id=instruction[1],entry=state.repeat.get(id);if(!entry)throw Error('Regex repeat frame missing');const repeat=code[instruction[2]];if(repeat[0]!=='repeat')throw Error('Regex repeat edge mismatch');if(entry.at===state.at&&entry.count>=repeat[2]){failed=true;break;}state=duplicate(state);state.repeat.set(id,{count:entry.count+1,at:entry.at});state.pc=instruction[2];continue;}
    default:throw Error('Unknown regex instruction');
   }
   if(failed){if(!fail())return null;}else state.pc++;
  }}finally{retained-=stack.length;}
 };
 const results:{from:number;to:number;captures:([number,number]|null)[]}[]=[];let search=0;
 while(search<=chars.length){let found:{from:number;to:number;captures:([number,number]|null)[]}|null=null;for(let at=search;at<=chars.length;at++){const result=run(program.code,at,Array(program.groups+1).fill(null),Array(program.groups+1).fill(null),1,0);if(result){found={from:at,to:result.at,captures:result.captures};found.captures[0]=[at,result.at];break;}if(program.flags.includes('y'))break;}if(!found)break;results.push(found);if(!program.flags.includes('g'))break;search=found.to===found.from?found.to+1:found.to;}
 const utf16=(at:number)=>offsets[at];
 return results.map(result=>({from:utf16(result.from),to:utf16(result.to),captures:result.captures.map(range=>range?text(chars.slice(...range),unicode):null),indices:result.captures.map(range=>range?[utf16(range[0]),utf16(range[1])]:null)}));
}

// Unicode17 TextMate format policy, using frozen data and bounded scans.
const regexV3FormatCache=new WeakMap<RegexV3CaseData,{upper:Map<number,string>;lower:Map<number,string>}>();
export function formatter(data:RegexV3CaseData,tick:()=>void=()=>{}){
 let maps=regexV3FormatCache.get(data);if(!maps){maps={upper:new Map(data.upper),lower:new Map(data.lower)};regexV3FormatCache.set(data,maps);}const {upper,lower}=maps;
 const contains=(ranges:number[][],point:number)=>{let low=0,high=ranges.length;while(low<high){tick();const mid=(low+high)>>>1;if(ranges[mid][0]<=point)low=mid+1;else high=mid;}return low>0&&point<=ranges[low-1][1];};
 const is=(kind:'letter'|'cased'|'ignorable'|'upper_letter'|'lower_letter'|'space',ch:string|undefined)=>ch!==undefined&&contains(data[kind],ch.codePointAt(0)!);
 const convert=(value:string,mode:string)=>{const chars=Array.from(value),map=mode==='upper'?upper:lower;return chars.map((ch,index)=>{tick();if(mode==='lower'&&ch==='Σ'){let before=index-1,after=index+1;while(before>=0&&is('ignorable',chars[before])){tick();before--;}while(after<chars.length&&is('ignorable',chars[after])){tick();after++;}if(before>=0&&is('cased',chars[before])&&(after===chars.length||!is('cased',chars[after])))return 'ς';}return map.get(ch.codePointAt(0)!)??ch;}).join('');};
 const digit=(ch:string|undefined)=>ch!==undefined&&ch>='0'&&ch<='9';
 const white=(ch:string|undefined)=>ch!==undefined&&(ch==='\t'||ch==='\n'||ch==='\v'||ch==='\f'||ch==='\r'||ch==='\uFEFF'||ch==='\u2028'||ch==='\u2029'||is('space',ch));
 const delimiter=(ch:string|undefined)=>ch==='_'||ch==='-'||white(ch);
 const words=(value:string)=>{const result=[];let current='';for(const ch of value){tick();if(is('letter',ch)||digit(ch))current+=ch;else if(current){result.push(current);current='';}}if(current)result.push(current);return result;};
 const kebab=(value:string)=>{const chars=Array.from(value);if(!chars.some(ch=>{tick();return is('letter',ch)||digit(ch);}))return value;let first=0,last=chars.length;while(first<last&&white(chars[first]))first++;while(last>first&&white(chars[last-1]))last--;while(first<last&&chars[first]==='_')first++;while(last>first&&chars[last-1]==='_')last--;const cleaned=chars.slice(first,last),parts=[];
  for(let at=0;at<cleaned.length;){tick();let end=at;
   let run=at;while(run<cleaned.length&&is('upper_letter',cleaned[run])){tick();run++;}
   for(let stop=run;stop-at>=2;stop--){tick();const ch=cleaned[stop];if(stop===cleaned.length||delimiter(ch)||is('upper_letter',ch)&&is('lower_letter',cleaned[stop+1])){end=stop;break;}}
   if(end===at){let stop=at;if(is('upper_letter',cleaned[stop]))stop++;const begin=stop;while(stop<cleaned.length&&is('lower_letter',cleaned[stop])){tick();stop++;}if(stop>begin){while(digit(cleaned[stop])){tick();stop++;}end=stop;}}
   if(end===at&&is('upper_letter',cleaned[at])&&is('upper_letter',cleaned[at+1])&&is('lower_letter',cleaned[at+2]))end=at+1;
   if(end===at&&is('upper_letter',cleaned[at])&&(at+1===cleaned.length||delimiter(cleaned[at+1])))end=at+1;
   if(end===at&&digit(cleaned[at])){end=at+1;while(digit(cleaned[end])){tick();end++;}}
   if(end>at){parts.push(convert(cleaned.slice(at,end).join(''),'lower'));at=end;}else at++;
  }
  if(parts.length)return parts.join('-');const fallback=[];let current='';for(const ch of cleaned){tick();if(delimiter(ch)){if(current){fallback.push(convert(current,'lower'));current='';}}else current+=ch;}if(current)fallback.push(convert(current,'lower'));return fallback.join('-');
 };
 const snake=(value:string)=>{const chars=Array.from(value),parts=[];let separator=false;for(let at=0;at<chars.length;at++){tick();const ch=chars[at];if(white(ch)||ch==='-'){if(!separator)parts.push('_');separator=true;continue;}separator=false;parts.push(ch);if(is('lower_letter',ch)&&is('upper_letter',chars[at+1]))parts.push('_');}return convert(parts.join(''),'lower');};
 return (value:string,mode:string)=>{switch(mode){case 'upcase':return convert(value,'upper');case 'downcase':return convert(value,'lower');case 'capitalize':return convert(value.slice(0,1),'upper')+value.slice(1);case 'camelcase':case 'pascalcase':{const found=words(value);return found.length?found.map((word,index)=>convert(word.slice(0,1),index===0&&mode==='camelcase'?'lower':'upper')+word.slice(1)).join(''):value;}case 'kebabcase':return kebab(value);case 'snakecase':return snake(value);default:return value;}};
}
export function appendBudget(maximum=1048576){let bytes=0,pending=false;const parts:string[]=[];return {append(value:string){for(let at=0;at<value.length;at++){const unit=value.charCodeAt(at);if(pending){if(unit<0xdc00||unit>0xdfff)throw Error('Regex result is not valid source Unicode');pending=false;bytes+=4;}else if(unit>=0xd800&&unit<=0xdbff){pending=true;}else if(unit>=0xdc00&&unit<=0xdfff)throw Error('Regex result is not valid source Unicode');else bytes+=unit<=127?1:unit<=2047?2:3;if(bytes>maximum)throw Error('Regex output budget exceeded');}parts.push(value);},finish(){if(pending)throw Error('Regex result is not valid source Unicode');return parts.join('');}};}
export function emitFormat(ops:RegexV3Format[],captures:(string|null)[],mode:(value:string,mode:string)=>string,append:(value:string)=>void,tick:()=>void,depth=0){if(depth>32)throw Error('Regex format depth budget exceeded');for(const op of ops){tick();if(typeof op==='string'){append(op);continue;}const value=captures[op[1]]??'';if(op[0]==='if')emitFormat(op[value?2:3],captures,mode,append,tick,depth+1);else append(mode(value,op[2]));}}

// Wire admission is separate from matching: malformed data never reaches the VM.
const keys=(value:unknown,names:string[]):value is Record<string,unknown> =>value!==null&&typeof value==='object'&&!Array.isArray(value)&&Object.keys(value).length===names.length&&Object.keys(value).every(key=>names.includes(key));
const fail:(message:string)=>never=(message)=>{throw Error(message);};
const integer=(raw:unknown,max:number):number=>typeof raw==='number'&&Number.isSafeInteger(raw)&&raw>=0&&raw<=max?raw:fail('Malformed regex integer');
const boolean=(raw:unknown):boolean=>typeof raw==='boolean'?raw:fail('Malformed regex boolean');
const list=(raw:unknown):unknown[]=>Array.isArray(raw)?raw:fail('Malformed regex array');
const scalar=(source:string):boolean=>{for(let at=0;at<source.length;at++){const cp=source.charCodeAt(at);if(cp>=0xd800&&cp<=0xdbff){const next=source.charCodeAt(++at);if(!(next>=0xdc00&&next<=0xdfff))return false;}else if(cp>=0xdc00&&cp<=0xdfff)return false;}return true;};
const admittedRegexV3=new WeakSet<object>();
const admittedRegexV3Bytes=new WeakMap<object,number>();
export function readRegexV3(raw:unknown):RegexV3Program{
 if(raw&&typeof raw==='object'&&admittedRegexV3.has(raw))return raw as RegexV3Program;
 if(!keys(raw,['version','case_map','unicode_map','groups','flags','code','subprograms','format'])||raw.version!==3||raw.case_map!=='hickory-ucd-17.0.0-v1'||raw.unicode_map!=='hickory-regex-ucd-17.0.0-v1')fail('Unsupported regex program');
 const groups=integer(raw.groups,32),flags=raw.flags;if(typeof flags!=='string'||flags.length>8||[...flags].some(ch=>!'dgimsuvy'.includes(ch))||new Set(flags).size!==flags.length||flags.includes('u')&&flags.includes('v'))fail('Malformed regex flags');const unicode=flags.includes('u')||flags.includes('v');
 let nodes=0,instructions=0;const tick=(n:number)=>{if(n<0||nodes>262144-n)fail('Regex metadata budget exceeded');nodes+=n;};const subs=list(raw.subprograms);if(subs.length>128)fail('Regex assertion budget exceeded');
 const indices=(value:unknown):number[]=>{const result=list(value).map(v=>integer(v,groups));if(result.length>groups||result.includes(0)||new Set(result).size!==result.length)fail('Malformed regex capture list');return result;};
 const code=(raw:unknown,direction:number)=>{const rows=list(raw);if(!rows.length||rows.length>4096||instructions>4096-rows.length)fail('Regex instruction budget exceeded');instructions+=rows.length;tick(rows.length);const pc=(v:unknown)=>integer(v,rows.length-1),group=(v:unknown)=>integer(v,groups)||fail('Regex group zero write');const result=rows.map(value=>{const op=list(value),kind=op[0];tick(op.length);const size=(n:number)=>{if(op.length!==n)fail('Malformed regex instruction');};switch(kind){
 case 'accept':size(1);return [kind];case 'jump':size(2);return [kind,pc(op[1])];case 'split':size(3);return [kind,pc(op[1]),pc(op[2])];case 'mark':case 'capture':size(2);return [kind,group(op[1])];
 case 'assert':size(4);if(typeof op[1]!=='string'||!['start','end','word','nonword'].includes(String(op[1]))||boolean(op[3])&&!unicode)fail('Malformed regex assertion');return [kind,op[1],boolean(op[2]),op[3]];
 case 'ref':size(4);if(boolean(op[3])!==unicode)fail('Regex reference mode mismatch');return [kind,indices(op[1]),boolean(op[2]),unicode];
 case 'look':size(4);if(!subs.length)fail('Missing regex assertion');return [kind,integer(op[1],subs.length-1),boolean(op[2]),indices(op[3])];
 case 'repeat':{size(8);const min=integer(op[2],Number.MAX_SAFE_INTEGER),max=op[3]===null?null:integer(op[3],Number.MAX_SAFE_INTEGER);if(max!==null&&max<min)fail('Backwards regex repeat');return [kind,integer(op[1],4095),min,max,boolean(op[4]),pc(op[5]),pc(op[6]),indices(op[7])];}
 case 'repeat_end':size(3);return [kind,integer(op[1],4095),pc(op[2])];
 case 'set':{size(5);if(boolean(op[4])!==unicode)fail('Regex set mode mismatch');const ranges=list(op[1]),strings=list(op[2]);if(ranges.length>65536||strings.length>65536)fail('Regex set budget');tick(ranges.length+strings.length);const maximum=unicode?0x10ffff:65535;let last=-2;const rs=ranges.map(raw=>{const pair=list(raw);if(pair.length!==2)fail('Malformed regex range');const low=integer(pair[0],maximum),high=integer(pair[1],maximum);if(low>high||last+1>=low)fail('Unnormalized regex ranges');last=high;return [low,high];});let prior:number[]|null=null;const ss=strings.map(raw=>{const chars=list(raw);if(chars.length>8192||chars.length===1)fail('Malformed regex set string');tick(chars.length);const value=chars.map(cp=>integer(cp,maximum));if(prior){let order=0;for(let i=0;i<Math.min(prior.length,value.length);i++)if(prior[i]!==value[i]){order=prior[i]-value[i];break;}if(prior.length<value.length||prior.length===value.length&&order>=0)fail('Unnormalized regex strings');}prior=value;return value;});return [kind,rs,ss,boolean(op[3]),unicode];}
 default:fail('Unknown regex instruction');}}) as RegexV3Instruction[];for(const op of result)if(op[0]==='repeat_end'&&(result[op[2]][0]!=='repeat'||result[op[2]][1]!==op[1]))fail('Regex repeat edge mismatch');return {direction,code:result};};
 const main=code(raw.code,1),subprograms=subs.map(sub=>{if(!keys(sub,['direction','code'])||typeof sub.direction!=="number"||![1,-1].includes(sub.direction))fail('Malformed regex subprogram');return code(sub.code,sub.direction);});
 const format=(raw:unknown,depth=0):RegexV3Format[]=>{const ops=list(raw);if(depth>32||ops.length>512)fail('Regex format budget');tick(ops.length);return ops.map(raw=>{if(typeof raw==='string'){if(raw.length>16384||!scalar(raw))fail('Regex format text budget');const bytes=new TextEncoder().encode(raw).length;if(bytes>16384)fail('Regex format text budget');tick(bytes);return raw;}const op=list(raw);if(op[0]==='capture'&&op.length===3&&typeof op[2]==='string'&&['','upcase','downcase','capitalize','camelcase','pascalcase','kebabcase','snakecase'].includes(String(op[2])))return ['capture',integer(op[1],groups+1),String(op[2])];if(op[0]==='if'&&op.length===4)return ['if',integer(op[1],groups+1),format(op[2],depth+1),format(op[3],depth+1)];fail('Unknown regex format');});};
 const result={version:3,case_map:raw.case_map,unicode_map:raw.unicode_map,groups,flags,code:main.code,subprograms,format:format(raw.format)};const byteCount=regexV3MetadataBytes(result);const freeze=(value:unknown):unknown=>{if(value&&typeof value==='object'){for(const child of Object.values(value))freeze(child);Object.freeze(value);}return value;};const admitted=freeze(result) as RegexV3Program;admittedRegexV3.add(admitted);admittedRegexV3Bytes.set(admitted,byteCount);return admitted;
}

export function applyRegexV3(program:RegexV3Program,input:string,data:RegexV3CaseData,budget:RegexV3Budget={work:0}):string {const matches=execute(program,input,casePolicy(data),budget),output=appendBudget(),tick=()=>{if(++budget.work>100000)throw Error('Regex format work budget exceeded');},mode=formatter(data,tick);if(!matches.length&&program.format.some(op=>typeof op!=='string'&&op[0]==='if'&&op[3].length)){emitFormat(program.format,[],mode,value=>output.append(value),tick);return output.finish();}let cursor=0;for(const found of matches){output.append(input.slice(cursor,found.from));emitFormat(program.format,found.captures,mode,value=>output.append(value),tick);cursor=found.to;}output.append(input.slice(cursor));return output.finish();}

function regexV3MetadataBytes(raw:unknown):number {let total=0;const add=(n:number)=>{if(n<0||total>1048576-n)throw Error('Regex metadata byte budget exceeded');total+=n;};const string=(value:string)=>{add(2);for(const char of value){const cp=char.codePointAt(0)!;add([34,92,8,9,10,12,13].includes(cp)?2:cp<32?6:cp<=127?1:cp<=2047?2:cp<=65535?3:4);}};const visit=(value:unknown,depth:number)=>{if(depth>128)throw Error('Regex metadata nesting exceeded');if(value===null)add(4);else if(typeof value==='boolean')add(value?4:5);else if(typeof value==='string')string(value);else if(typeof value==='number'){if(!Number.isSafeInteger(value))throw Error('Malformed regex metadata integer');add(String(value).length);}else if(Array.isArray(value)){add(2+Math.max(0,value.length-1));for(const child of value)visit(child,depth+1);}else if(value&&typeof value==='object'){const entries=Object.entries(value);add(2+Math.max(0,entries.length-1));for(const [key,child]of entries){string(key);add(1);visit(child,depth+1);}}else throw Error('Malformed regex metadata value');};visit(raw,0);return total;}

export const regexV3ProgramBytes=(program:RegexV3Program):number=>admittedRegexV3Bytes.get(readRegexV3(program))!;
const frozenRegexV3Case=frozenCase as unknown as RegexV3CaseData;
export const evaluateRegexV3=(program:RegexV3Program,input:string,budget:RegexV3Budget={work:0})=>applyRegexV3(program,input,frozenRegexV3Case,budget);
