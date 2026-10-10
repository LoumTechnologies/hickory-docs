# Hickory Docs

Hickory Docs is a new IDE sitting at the intersection between literate programming, AI coding agents and Jupyter notebooks.

You can write files like this:

<hick:file path="greeting.txt">
<hick:copy id="greeting">Hello from Hickory Docs!
</hick:copy>
<hick:paste select="#greeting" />
</hick:file>

`hick:copy` makes it so you can reuse its contents using `hick:paste` elements. It's like copy and paste but with css selectors to selector which thing to paste. 

You can execute code like this:

<hick:container name="shell" image="alpine:3.20" />

<hick:exec id="hello-and-math" container="shell" show="output">
printf 'Hello from Hickory Docs!\n'
printf '2 + 3 = %s\n' "$((2 + 3))"
<hick:expect match="exact">Hello from Hickory Docs!
2 + 3 = 5
</hick:expect>
</hick:exec>

Running the cell displays its output in the document. The `hick:expect` block records the expected result, so verification can catch a change in behavior. The explanation, executable commands, and expected output all live together.
