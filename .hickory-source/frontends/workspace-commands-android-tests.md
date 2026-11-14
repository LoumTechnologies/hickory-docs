<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Recheck commands after context changes

For maintainers changing workspace services independently of the editor renderer.

A search result may outlive the workspace state that enabled it. Resolve Save once, change the state and invoke that retained command: it must refuse while saving, read-only or without an editor, then use current eligibility when valid again. Unknown requirements and missing adapters must not become enabled. Native witnesses load the same bundled descriptors as production, catching packaging failures without opening an editor.

<ui:copy id="simple-workspace-commands-android-tests">
package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
class WorkspaceCommandsTest {
    @Test fun bundledCommandsRecheckContext() {
        val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use {it.readText()});assertTrue(descriptors.isNotEmpty())
        var context=CommandContext(true,true,true,false,false,false,true,1);var called=0
        val commands=workspaceCommands({context},mapOf("file.save" to {called++;Unit}),descriptors);val save=commands.first {it.id=="file.save"};assertNull(save.reason)
        context=context.copy(saving=true);save.run();context=context.copy(saving=false,writable=false);save.run();context=context.copy(writable=true,editor=false);save.run();assertEquals(0,called)
        context=context.copy(editor=true);save.run();assertEquals(1,called);assertNotNull(commands.first {it.id=="file.open"}.reason)
        assertNotNull(commandReason("unknown",context));assertNotNull(commandReason("back",context));assertNotNull(commandReason("multi",context))
    }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/WorkspaceCommandsTest.kt"><ui:paste select="#simple-workspace-commands-android-tests"/></ui:file>
</ui:doc>
