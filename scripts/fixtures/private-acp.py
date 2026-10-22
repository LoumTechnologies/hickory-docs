#!/usr/bin/env python3
"""Scripted ACP protocol witness. No model/provider calls or semantic judgment."""
import json
import sys
import urllib.request

url = None
calls = json.load(open(sys.argv[1]))
for line in sys.stdin:
    message = json.loads(line)
    method = message['method']
    if method == 'initialize':
        result = {'protocolVersion': 1, 'agentCapabilities': {'mcpCapabilities': {'http': True}}}
    elif method == 'session/new':
        url = message['params']['mcpServers'][0]['url']
        result = {'sessionId': 'scripted-private-acp'}
    elif method == 'session/prompt':
        for name, arguments in calls:
            request = urllib.request.Request(url, json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call', 'params': {'name': name, 'arguments': arguments}}).encode(), {'Content-Type': 'application/json'})
            with urllib.request.urlopen(request, timeout=90) as response:
                value = json.load(response)
            if 'result' not in value:
                raise RuntimeError('Private fixture did not receive an MCP tool result')
        result = {'stopReason': 'end_turn'}
    else:
        raise RuntimeError('Unexpected ACP fixture method')
    print(json.dumps({'jsonrpc': '2.0', 'id': message['id'], 'result': result}), flush=True)
