<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# scripts/fixtures/private-acp.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `4127d81802d763949bd41be14928fa750ffe41c71bed244c4740125cb5ee9528`.

<book:entry path="scripts/fixtures/private-acp.py">
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
</book:entry>

</book:doc>
