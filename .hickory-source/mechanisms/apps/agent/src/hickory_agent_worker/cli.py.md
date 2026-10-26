<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/agent/src/hickory_agent_worker/cli.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f97597b8a030"></a>
## `apps/agent/src/hickory_agent_worker/cli.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

<a id="f-f97597b8a030-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-f97597b8a030-expression_statement">
"""Standalone development CLI. This mode provides no hosted confinement claim."""
</storycode:copy>
<a id="f-f97597b8a030-argparse"></a>
### `argparse`

<storycode:copy id="f-f97597b8a030-argparse">
import argparse
</storycode:copy>
<a id="f-f97597b8a030-importlib"></a>
### `importlib`

<storycode:copy id="f-f97597b8a030-importlib">
import importlib
</storycode:copy>
<a id="f-f97597b8a030-json"></a>
### `json`

<storycode:copy id="f-f97597b8a030-json">
import json
</storycode:copy>
<a id="f-f97597b8a030-os"></a>
### `os`

<storycode:copy id="f-f97597b8a030-os">
import os
</storycode:copy>
<a id="f-f97597b8a030-urllib-request"></a>
### `urllib.request`

<storycode:copy id="f-f97597b8a030-urllib-request">
import urllib.request
</storycode:copy>
<a id="f-f97597b8a030-sys"></a>
### `sys`

<storycode:copy id="f-f97597b8a030-sys">
import sys
</storycode:copy>
<a id="f-f97597b8a030-tempfile"></a>
### `tempfile`

<storycode:copy id="f-f97597b8a030-tempfile">
import tempfile
</storycode:copy>
<a id="f-f97597b8a030-Path"></a>
### `Path`

<storycode:copy id="f-f97597b8a030-Path">
from pathlib import Path

</storycode:copy>
<a id="f-f97597b8a030-execute"></a>
### `execute`

<storycode:copy id="f-f97597b8a030-execute">
from .runtime import execute
</storycode:copy>
<a id="f-f97597b8a030-Agent"></a>
### `Agent`

<storycode:copy id="f-f97597b8a030-Agent">
from .core import Agent, Config
</storycode:copy>
<a id="f-f97597b8a030-LIMITS"></a>
### `LIMITS`

<storycode:copy id="f-f97597b8a030-LIMITS">
from .hosted import LIMITS, monty


</storycode:copy>
<a id="f-f97597b8a030-ReplayModel"></a>
### `ReplayModel`

<storycode:copy id="f-f97597b8a030-ReplayModel">
class ReplayModel:
    def __init__(self, responses):
        self.responses = iter(responses)

    def complete(self, messages):
        try:
            response = next(self.responses)
        except StopIteration:
            raise RuntimeError('Fake model responses exhausted') from None
        return response if isinstance(response, str) else json.dumps(response)


</storycode:copy>
<a id="f-f97597b8a030-HttpModel"></a>
### `HttpModel`

<storycode:copy id="f-f97597b8a030-HttpModel">
class HttpModel:
    """Minimal OpenAI-compatible development client; credentials stay in this process."""
    def __init__(self, model, base_url, api_key):
        self.model, self.url, self.api_key = model, base_url.rstrip('/') + '/chat/completions', api_key
        if not api_key:
            raise ValueError('Set OPENAI_API_KEY, or use --responses for offline development')

    def complete(self, messages):
        request = urllib.request.Request(self.url,
            data=json.dumps({'model': self.model, 'messages': messages}).encode(),
            headers={'Content-Type': 'application/json', 'Authorization': 'Bearer ' + self.api_key})
        with urllib.request.urlopen(request, timeout=180) as response:
            payload = json.loads(response.read(1_048_577))
        return payload['choices'][0]['message']['content']


</storycode:copy>
<a id="f-f97597b8a030-DevelopmentTools"></a>
### `DevelopmentTools`

<storycode:copy id="f-f97597b8a030-DevelopmentTools">
class DevelopmentTools:
    """Read-only fixture tools. No real workspace mutation or verification claim."""
    catalogue = {'tools': [
        {'name': 'list_docs', 'description': 'List Markdown fixture paths, read-only.',
         'inputSchema': {'type': 'object', 'properties': {}}},
        {'name': 'read_doc', 'description': 'Read a fixture document, read-only.',
         'inputSchema': {'type': 'object', 'properties': {'doc': {'type': 'string'}}, 'required': ['doc']}},
    ]}

    def __init__(self, workspace):
        self.workspace = Path(workspace).resolve()

    def call(self, name, arguments):
        if name == 'list_docs':
            return sorted(str(p.relative_to(self.workspace))
                          for p in self.workspace.rglob('*.md') if not p.is_symlink())
        if name == 'read_doc':
            path = (self.workspace / arguments['doc']).resolve()
            if not path.is_relative_to(self.workspace):
                raise ValueError('Path is outside the development workspace')
            return path.read_text()
        raise ValueError(f'Development tool {name!r} unavailable; inject a tool client to test it')

    def complete(self, summary):
        return {'accepted': True, 'evidence': 'development-only; no Hickory verification'}


</storycode:copy>
<a id="f-f97597b8a030-ToolBridge"></a>
### `ToolBridge`

<storycode:copy id="f-f97597b8a030-ToolBridge">
class ToolBridge:
    def __init__(self, tools):
        self.tools = tools

    def call(self, method, params):
        if method != 'tool':
            raise ValueError('Only tool callbacks are available to snippets')
        return self.tools.call(params['name'], params['arguments'])


</storycode:copy>
<a id="f-f97597b8a030-main"></a>
### `main`

<storycode:copy id="f-f97597b8a030-main">
def main(argv=None):
    parser = argparse.ArgumentParser(description='Run the Python agent independently (development mode)')
    parser.add_argument('prompt')
    choice = parser.add_mutually_exclusive_group(required=True)
    choice.add_argument('--responses', type=Path, help='JSON array of fake model responses; no network')
    choice.add_argument('--api-model', help='Model at an OpenAI-compatible endpoint (makes paid calls)')
    parser.add_argument('--base-url', default=os.environ.get('OPENAI_BASE_URL', 'https://api.openai.com/v1'))
    choice.add_argument('--factory', help='Trusted module:function returning (model_client, tool_client) for workspace')
    parser.add_argument('--workspace', type=Path, help='Read-only fixture directory; defaults to a disposable directory')
    parser.add_argument('--max-turns', type=int, default=20)
    args = parser.parse_args(argv)
    try:
        with tempfile.TemporaryDirectory(prefix='hickory-agent-dev-') as scratch:
            workspace = args.workspace or Path(scratch)
            if not workspace.is_dir():
                raise ValueError(f'Development workspace does not exist: {workspace}')
            if args.factory:
                module, function = args.factory.split(':', 1)
                model, tools = getattr(importlib.import_module(module), function)(workspace)
            elif args.api_model:
                model = HttpModel(args.api_model, args.base_url, os.environ.get('OPENAI_API_KEY'))
                tools = DevelopmentTools(workspace)
            else:
                responses = json.loads(args.responses.read_text())
                if not isinstance(responses, list):
                    raise ValueError('--responses must contain a JSON array')
                model, tools = ReplayModel(responses), DevelopmentTools(workspace)
            bridge = ToolBridge(tools)
            print(f'Development mode: {workspace}; no hosted confinement or verification claim', file=sys.stderr)
            with monty() as pool:
                with pool.checkout(limits=LIMITS) as session:
                    agent = Agent(model, tools, lambda code: execute(session, bridge, code),
                        lambda kind, data: print(json.dumps({'kind': kind, 'data': data}), file=sys.stderr))
                    print(agent.run(Config(prompt=args.prompt, max_turns=args.max_turns,
                        tools=getattr(tools, 'catalogue', {}))))
        return 0
    except KeyboardInterrupt:
        print('Agent cancelled', file=sys.stderr)
        return 130
    except Exception as exc:
        print(f'agent: {exc}', file=sys.stderr)
        return 1


</storycode:copy>
<a id="f-f97597b8a030-if_statement"></a>
### `if_statement`

<storycode:copy id="f-f97597b8a030-if_statement">
if __name__ == '__main__':
    raise SystemExit(main())
</storycode:copy>

### Assemble `apps/agent/src/hickory_agent_worker/cli.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/src/hickory_agent_worker/cli.py"><storycode:paste select="#f-f97597b8a030-expression_statement"/><storycode:paste select="#f-f97597b8a030-argparse"/><storycode:paste select="#f-f97597b8a030-importlib"/><storycode:paste select="#f-f97597b8a030-json"/><storycode:paste select="#f-f97597b8a030-os"/><storycode:paste select="#f-f97597b8a030-urllib-request"/><storycode:paste select="#f-f97597b8a030-sys"/><storycode:paste select="#f-f97597b8a030-tempfile"/><storycode:paste select="#f-f97597b8a030-Path"/><storycode:paste select="#f-f97597b8a030-execute"/><storycode:paste select="#f-f97597b8a030-Agent"/><storycode:paste select="#f-f97597b8a030-LIMITS"/><storycode:paste select="#f-f97597b8a030-ReplayModel"/><storycode:paste select="#f-f97597b8a030-HttpModel"/><storycode:paste select="#f-f97597b8a030-DevelopmentTools"/><storycode:paste select="#f-f97597b8a030-ToolBridge"/><storycode:paste select="#f-f97597b8a030-main"/><storycode:paste select="#f-f97597b8a030-if_statement"/></storycode:file>

</storycode:doc>
