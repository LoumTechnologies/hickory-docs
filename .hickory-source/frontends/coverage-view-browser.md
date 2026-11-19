<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read runner results and coverage on browser

For maintainers changing workspace services independently of the editor renderer.

The existing menu opens a floating, replaceable tool surface. The editor remains mounted and retains its own selection, scroll and undo state. A native local-only web renderer delegates only fixed coverage operations to the connected engine; workspace replacement retires its replies. Source navigation opens unbound files without positional claims, and accepts a reported line only after matching its captured hash against available text. Selecting a report retains its record identity; source-line mark binding is a separate capability.

<ui:copy id="simple-coverage-browser">
import {createPortal} from 'react-dom';
import {Component,useEffect,useMemo,type ReactNode} from 'react';
import {CoveragePanel} from '../coverage/CoverageTools';
import {coverageCall} from '../coverage/client';
import {selectCoverage,selectedCoverage} from '../coverage/selection';
import type {CoverageSurfaceClient} from '../coverage/CoverageSurface';
class CoverageBoundary extends Component<{children:ReactNode;close:()=>void},{failed:boolean}>{state={failed:false};static getDerivedStateFromError(){return {failed:true};}render(){return this.state.failed?<section className="simple-workspace-problems" role="alert">Coverage could not render. Close and reopen it.<button onClick={this.props.close}>Close</button></section>:this.props.children;}}
export function SimpleCoverageView({visible,workspace,path,language,close,navigate}:{visible:boolean;workspace:string;path:string|null;language:string;close:(navigated?:boolean)=>void;navigate:CoverageSurfaceClient["navigate"]}){
  useEffect(()=>{selectCoverage(null);return()=>selectCoverage(null);},[workspace]);
  const client=useMemo<CoverageSurfaceClient>(()=>({call:coverageCall,select:selectCoverage,navigate}),[navigate]);
  if(!visible)return null;
  return createPortal(<CoverageBoundary close={close}><CoveragePanel initialPath={path??'.'} initialLanguage={language} initialSelectedId={selectedCoverage()?.id} client={client} close={close}/></CoverageBoundary>,document.body);
}
</ui:copy>

<ui:file path="apps/web/src/simple/SimpleCoverageView.tsx"><ui:paste select="#simple-coverage-browser"/></ui:file>
</ui:doc>
