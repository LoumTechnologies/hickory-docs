import data from '../../../simple-contract/snippet-case-map.json';
export const snippetCaseVersion = 'hickory-ucd-16.0.0-v1';
const maps = {upper:new Map<number,string>(data.upper as [number,string][]),lower:new Map<number,string>(data.lower as [number,string][])};
function includes(ranges:number[][], scalar:number):boolean {let lo=0,hi=ranges.length;while(lo<hi){const mid=(lo+hi)>>>1;const r=ranges[mid];if(scalar<r[0])hi=mid;else if(scalar>r[1])lo=mid+1;else return true}return false}
export function snippetCaseContains(kind:'letter'|'identifier'|'cased'|'ignorable', scalar:number):boolean{return includes(data[kind],scalar)}
export function snippetCase(value:string, mode:'upper'|'lower'):string {
 const chars=Array.from(value);return chars.map((ch,index)=>{const cp=ch.codePointAt(0)!;if(mode==='lower'&&cp===0x3A3){let before=index-1,after=index+1;while(before>=0&&snippetCaseContains('ignorable',chars[before].codePointAt(0)!))before--;while(after<chars.length&&snippetCaseContains('ignorable',chars[after].codePointAt(0)!))after++;if(before>=0&&snippetCaseContains('cased',chars[before].codePointAt(0)!)&&(after===chars.length||!snippetCaseContains('cased',chars[after].codePointAt(0)!)))return 'ς'}return maps[mode].get(cp)??ch}).join('')
}
export function snippetCaseWords(value:string):string[]{const words:string[]=[];let current='';for(const ch of value){const cp=ch.codePointAt(0)!;if(snippetCaseContains('letter',cp)||cp>=48&&cp<=57)current+=ch;else if(current){words.push(current);current=''}}if(current)words.push(current);return words}
