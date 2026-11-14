package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
@Composable fun WorkspaceAnalysisView(workspace:Workspace) {
    val model=workspace.analysis;var query by remember {mutableStateOf("")};var page by remember {mutableStateOf(0)}
    val reports=model.knowledge.values.filter {(it.path+" "+it.notice).contains(query,ignoreCase=true)}.sortedBy {it.path}
    val last=maxOf(0,(reports.size+99)/100-1);val selected=minOf(page,last)
    Column(Modifier.widthIn(min=260.dp,max=380.dp).heightIn(max=420.dp).verticalScroll(rememberScrollState()).padding(8.dp)) {
        Text("Workspace analysis",fontSize=16.sp)
        Text("${model.completed}/${model.scope.files.size} attempted · ${model.knowledge.values.count {it.source!=null}} sources read · ${model.scope.generated} generated and ${model.scope.unclassified} unclassified excluded.",fontSize=11.sp)
        Text("Provider symbols and diagnostics arrive independently. Zero symbols may mean no declarations, unsupported analysis or indexing. This is not verification.",fontSize=11.sp)
        Row {TextButton(enabled=!model.running,onClick={workspace.startAnalysis()}) {Text("Read workspace")};TextButton(enabled=model.running&&!model.stopping,onClick={model.stop()}) {Text(if(model.stopping)"Stopping…" else "Stop")}}
        OutlinedTextField(query,{query=it;page=0},label={Text("Filter analysis")},singleLine=true)
        for(item in reports.drop(selected*100).take(100)) {Text(item.path,fontSize=12.sp);Text("${if(item.source!=null && workspace.buffers[item.path]?.text!=item.source) "stale" else item.state} · ${item.locations.size} symbols · ${item.notice}",fontSize=11.sp)}
        if(last>0) Row {TextButton(enabled=selected>0,onClick={page=selected-1}) {Text("Previous results")};Text("${selected+1}/${last+1}");TextButton(enabled=selected<last,onClick={page=selected+1}) {Text("Next results")}}
    }
}
