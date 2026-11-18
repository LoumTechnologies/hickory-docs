import {Worker,isMainThread,parentPort,workerData} from 'node:worker_threads';
import {readFile,writeFile} from 'node:fs/promises';
import {pathToFileURL} from 'node:url';
const root=process.argv[2]?pathToFileURL(process.argv[2].replace(/\/$/,'')+'/'):null;
if(isMainThread&&!root)throw Error('Pass the prepared Unicode17 scratch directory');
if(process.version!=='v24.22.0'||process.versions.unicode!=='17.0'||process.versions.icu!=='78.3')throw Error('Pinned Unicode17 Node runtime required');
if(!isMainThread){const output={};for(const property of workerData){let regex;try{regex=new RegExp('\\p{'+property+'}','u');}catch(error){output[property]={oracle_error:error.message};continue;}const ranges=[];let first=-1;for(let cp=0;cp<=0x110000;cp++){const yes=cp<0x110000&&regex.test(String.fromCodePoint(cp));if(yes&&first<0)first=cp;if(!yes&&first>=0){ranges.push([first,cp-1]);first=-1;}}output[property]=ranges;}parentPort.postMessage(output);}
else{const spec=JSON.parse(await readFile(new URL('property-requests.json',root))),count=4,chunks=Array.from({length:count},(_,index)=>spec.requests.filter((_,i)=>i%count===index));
const groups=await Promise.all(chunks.map(chunk=>new Promise((resolve,reject)=>{const worker=new Worker(new URL(import.meta.url),{workerData:chunk});worker.on('message',resolve);worker.on('error',reject);})));const properties=Object.assign({},...groups);
const upper=[],lower=[];for(let cp=0;cp<0x110000;cp++){if(cp>=0xd800&&cp<=0xdfff)continue;const ch=String.fromCodePoint(cp),up=ch.toUpperCase(),down=ch.toLowerCase();if(up!==ch)upper.push([cp,up]);if(down!==ch)lower.push([cp,down]);}
await writeFile(new URL('properties-17.json',root),JSON.stringify({version:'hickory-regex-ucd-17.0.0-v1',unicode:'17.0.0',oracle:{node:process.version,icu:process.versions.icu},aliases:spec.aliases,properties})+'\n');
await writeFile(new URL('case-17-partial.json',root),JSON.stringify({version:'hickory-ucd-17.0.0-v1',unicode:'17.0.0',upper,lower,letter:properties['General_Category=Letter'],cased:properties.Cased,ignorable:properties.Case_Ignorable,identifier:properties.XID_Continue})+'\n');console.log('Exported '+Object.keys(properties).length+' pinned Unicode17 property sets and case mappings');}
