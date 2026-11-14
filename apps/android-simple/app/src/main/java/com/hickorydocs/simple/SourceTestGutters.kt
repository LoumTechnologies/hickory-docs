package com.hickorydocs.simple
fun DocumentBuffer.readTestSources(){testIndex?.enqueue(file.path,text){text}}
fun DocumentBuffer.sourceTestRows()=indexedTestRows(testIndex?.current(file.path,text))
suspend fun Workspace.inspectTestSource(buffer:DocumentBuffer,line:Int) {
    val rows=buffer.sourceTestRows().filter {it.first==line}
    if(rows.isEmpty()){status="Source changed. Wait for test inspection or use Source tests in Edit.";return}
    val index=sourceTestIndex;val source=buffer.text
    open(buffer.file)
    if(sourceTestIndex!==index||buffer.text!=source||currentBuffer!==buffer)return
    val model=testSources;model.query=if(rows.size==1) rows[0].second.name else "";showTests=true
    model.refresh(this)
}
