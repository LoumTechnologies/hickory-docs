<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Inspect source tests from the existing gutter on android

For maintainers changing workspace services independently of the editor renderer.

The editor supplies text immediately; a separate bounded source index reads test declarations in the background. A source-bound hint uses the existing right-hand action column and overflow grid. Clicking it activates the existing buffer, then refreshes the full floating test view with current drafts before offering execution. Cached gutter facts never authorize a run. Other saved construction contributors can differ from retained drafts, so this fresh inspection is required even when the primary text still matches.

<ui:copy id="source-test-gutters-android">
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
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SourceTestGutters.kt"><ui:paste select="#source-test-gutters-android"/></ui:file>
</ui:doc>
