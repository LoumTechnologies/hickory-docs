import {createPortal} from 'react-dom';
import {Component,useEffect,useMemo,type ReactNode} from 'react';
import {CoveragePanel} from '../coverage/CoverageTools';
import {coverageCall} from '../coverage/client';
import {selectCoverage,selectedCoverage} from '../coverage/selection';
import type {CoverageSourceNavigation,CoverageSurfaceClient} from '../coverage/CoverageSurface';
class CoverageBoundary extends Component<{children:ReactNode;close:()=>void},{failed:boolean}>{state={failed:false};static getDerivedStateFromError(){return {failed:true};}render(){return this.state.failed?<section className="simple-workspace-problems" role="alert">Coverage could not render. Close and reopen it.<button onClick={this.props.close}>Close</button></section>:this.props.children;}}
export function SimpleCoverageView({visible,workspace,path,language,close,navigate}:{visible:boolean;workspace:string;path:string|null;language:string;close:()=>void;navigate:(location:CoverageSourceNavigation)=>Promise<void>}){
  useEffect(()=>{selectCoverage(null);return()=>selectCoverage(null);},[workspace]);
  const client=useMemo<CoverageSurfaceClient>(()=>({call:coverageCall,select:selectCoverage,navigate}),[navigate]);
  if(!visible)return null;
  return createPortal(<CoverageBoundary close={close}><CoveragePanel initialPath={path??'.'} initialLanguage={language} initialSelectedId={selectedCoverage()?.id} client={client} close={close}/></CoverageBoundary>,document.body);
}
