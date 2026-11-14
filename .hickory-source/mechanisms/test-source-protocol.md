<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Describe source-test facts on the wire

For maintainers changing workspace services independently of the editor renderer.

The engine returns separate authored and generated positions, each with its exact source fingerprint. A missing authored mapping stays null. Source confidence and name filters never become confirmed case IDs. Protocol version 1 distinguishes local constructions from an explicitly selected project closure, reports unused draft inputs, and can return a memory-only capture identity without executing a runner. These DTOs are consumed by the generated HTTP facade and shared UI adapters.

<ui:copy id="test-source-protocol-types">
import type { TerminalSession } from "./types";
/** Source-declared candidates are not confirmed runner case IDs or results. */
export interface TestSourcePoint {path:string;line:number;character:number;hash:string}
export interface SourceTestCandidate {id:string;language:string;run_scope:"file"|"file_filter"|"crate"|"package"|"project";name:string;name_filter:string|null;confidence:"syntax"|"heuristic"|"dynamic";source:TestSourcePoint|null;target:TestSourcePoint}
export interface SourceTestRequest {path:string;source:string;project_entry?:string;sources?:Record<string,string>;capture?:boolean}
export interface SourceTestScan {
  version:1;workspace:string;workspace_id:string;input_path:string;input_hash:string;scope:"plain"|"document"|"project";project_entry:string|null;fingerprint:string;
  source_hashes:Record<string,string>;candidates:SourceTestCandidate[];syntax_complete:boolean;notices:string[];unsupported_languages:string[];unclassified_outputs:string[];scanned_files:number;unused_sources:string[];
  project_context:{id:string;uri:string;immutable:true;materialized:false}|null;
}

export interface SourceTestRunRequest {version:1;input:{kind:"saved"};analysis:SourceTestRequest;fingerprint:string;candidate_id:string;selection:{kind:"containing_scope"|"name_filter"}}
export interface SourceTestRunResult extends TerminalSession {
  version:1;source_hash:string;source_checked_at_admission:true;source_frozen:false;checked_sources:Record<string,string>;analysis_fingerprint:string;candidate_id:string;candidate_name:string;selection:{kind:"containing_scope"|"name_filter"};
}
</ui:copy>

<ui:file path="apps/web/src/api/testSources.ts"><ui:paste select="#test-source-protocol-types"/></ui:file>
</ui:doc>
