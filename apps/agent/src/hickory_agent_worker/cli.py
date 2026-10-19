"""Standalone development CLI. This mode provides no hosted confinement claim."""
import argparse
import importlib
import json
import os
import urllib.request
import sys
import tempfile
from pathlib import Path

from .runtime import execute
from .core import Agent, Config
from .hosted import LIMITS, monty


class ReplayModel:
    def __init__(self, responses):
        self.responses = iter(responses)

    def complete(self, messages):
        try:
            response = next(self.responses)
        except StopIteration:
            raise RuntimeError('Fake model responses exhausted') from None
        return response if isinstance(response, str) else json.dumps(response)


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


class ToolBridge:
    def __init__(self, tools):
        self.tools = tools

    def call(self, method, params):
        if method != 'tool':
            raise ValueError('Only tool callbacks are available to snippets')
        return self.tools.call(params['name'], params['arguments'])


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


if __name__ == '__main__':
    raise SystemExit(main())
