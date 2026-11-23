<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Run and test from the pane menu without adding a toolbar row

For: maintainers and authors selecting the current editor or debugger behavior.

Open the document menu while a run is active. Run and Test must refer to that document and stay disabled until the run permits another action.

## What must happen

Run executes the document; Test matches the hick test operation. These document-wide actions use the pane-header menu, while per-cell actions remain on the editor rail. Run and Test are disabled while a document run is in flight. A baseline action is named for the output baseline it establishes. The problem count opens the cross-document problem list; F8 retains next-problem navigation.

## Limits of the evidence

The focused pane test uses injected session callbacks. It does not establish native menu placement, the runtime test outcome or every problem-list interaction.



## Implementation

</storycode:doc>
