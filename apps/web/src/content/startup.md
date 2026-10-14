# Hickory Docs

Keep the explanation, source code and executable checks together in one Markdown document.

Save this note as a `.md` file. Click the ▶ beside the command cell below to run it. Open the command palette with Cmd/Ctrl+Shift+P and choose **Test document** to check its expected output.

## Generate a file

This block writes `greeting.txt` beside the saved note:

<hick:file path="greeting.txt">
<hick:copy id="greeting">Hello from Hickory Docs!
</hick:copy>
<hick:paste select="#greeting" />
</hick:file>

The named `hick:copy` supplies the same text to `hick:paste`. Edit it once to change both copies. Generated files carry their edits back to the source document while Hickory is watching the folder.

## Run a check

<hick:container name="shell" image="alpine:3.20" />

<hick:exec id="hello-and-math" container="shell" show="output">
printf 'Hello from Hickory Docs!\n'
printf '2 + 3 = %s\n' "$((2 + 3))"
<hick:expect match="exact">Hello from Hickory Docs!
2 + 3 = 5
</hick:expect>
</hick:exec>

Running displays the result here. `hick:expect` records what the output should be; Test checks it by running the commands again.

## Work with an agent

Choose **Agent** in the command palette. Type a request in **Your response** and send with Cmd/Ctrl+Enter. The pane names the open editors it will include as context. Choose an agent in Agent settings; each session is recorded as a `.md` document.
