export interface CoverageProjectionInput {
  id:string;source_changed_during_run?:boolean;identity:{source_hashes:Record<string,string>};
  evidence?:{report:{files:Record<string,{line_counts:boolean;lines:Record<string,number>}>}}|null;
  mappings?:Record<string,Record<string,{path:string;line:number}>>;
}
export interface CoverageProjectedLine {line:number;state:"covered"|"uncovered"|"mixed";count:number|null;origins:string[]}
/** One authored line may represent several runtime locations; retain mixed evidence. */
export function coverageProjection(record:CoverageProjectionInput):Record<string,CoverageProjectedLine[]> {
  if(record.source_changed_during_run!==false)return {};
  type Aggregate={hit:boolean;miss:boolean;count:number|null;origins:string[]};
  const owners=new Map<string,Map<number,Aggregate>>();
  const add=(path:string,line:number,count:number,available:boolean,origin:string)=>{
    if(!/^[0-9a-f]{64}$/i.test(record.identity.source_hashes[path]??"")||!Number.isInteger(line)||line<1||line>2147483647||!Number.isFinite(count)||count<0)return;
    const rows=owners.get(path)??new Map<number,Aggregate>();owners.set(path,rows);
    const known=available&&Number.isSafeInteger(count)?count:null,old=rows.get(line-1);
    rows.set(line-1,{hit:(old?.hit??false)||count>0,miss:(old?.miss??false)||count===0,count:old?(old.count===null||known===null?null:Math.min(old.count,known)):known,origins:[...(old?.origins??[]),origin]});
  };
  for(const [path,file] of Object.entries(record.evidence?.report.files??{}))for(const [value,count] of Object.entries(file.lines)){
    const line=Number(value),origin=`${path}:${value}`;add(path,line,count,file.line_counts,origin);
    const mapping=record.mappings?.[path]?.[value];if(mapping&&mapping.path!==path)add(mapping.path,mapping.line,count,file.line_counts,origin);
  }
  return Object.fromEntries([...owners].map(([path,rows])=>[path,[...rows].sort(([a],[b])=>a-b).map(([line,row])=>({line,state:row.hit&&row.miss?"mixed" as const:row.hit?"covered" as const:"uncovered" as const,count:row.count,origins:row.origins.sort()}))]));
}
