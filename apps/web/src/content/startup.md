# Hickory Docs

Hickory Docs is a new IDE that is a cross between literate programming, AI coding agents and Jupyter notebooks. It uses Markdown files.

It supports debugging and intellisense features within the Markdown editor for many popular programming languages via LSP and DAP.

Write files like this:

<hick:copy id="greeting">
Hello from Hickory Docs!
</hick:copy>

<hick:file path="test.py">
println("""<hick:paste select="#greeting" />""")
</hick:file>

Three elements are involved here:

- `hick:copy`
- `hick:paste`
- `hick:file`

When you process this file with Hickory Docs, it creates `test.py`:

```python
println("""
Hello from Hickory Docs!
""")
```

### Executing Code in a Notebook

You can also execute code like this:

<hick:exec id="hello-and-math" container="shell" show="output">
printf 'Hello from Hickory Docs!\n'
printf '2 + 3 = %s\n' "$((2 + 3))"
</hick:exec>
