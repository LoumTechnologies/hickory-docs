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
class TemplateCommandTest {
 @Test fun menusRecheckCurrentFieldsChoicesAndWriteAvailability(){
  var context=CommandContext(true,true,true,false,false,false,true,1);var called=0
  assertNotNull(commandReason("template",context));context=context.copy(snippet=true);assertNull(commandReason("template",context));assertNotNull(commandReason("templateChoice",context));context=context.copy(snippetChoices=true);assertNull(commandReason("templateChoice",context));context=context.copy(saving=true);assertNotNull(commandReason("templateChoice",context));assertNull(commandReason("template",context));context=context.copy(saving=false)
  val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()});val command=workspaceCommands({context},mapOf("editor.snippetNext" to {called++;Unit}),descriptors).first{it.id=="editor.snippetNext"};assertNull(command.reason);context=context.copy(snippet=false);command.run();assertEquals(0,called)
 }
}
