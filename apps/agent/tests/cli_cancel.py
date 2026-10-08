"""Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md.
Drive the real CLI's terminal review with a local provider fixture; cancel, never approve.
"""
import http.server
import json
import os
import pty
import select
import signal
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path


class Provider(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        assert self.headers['Authorization'] == 'Bearer fixture-not-a-real-key'
        self.rfile.read(int(self.headers['Content-Length']))
        body = json.dumps({
            'choices': [{'message': {'content': json.dumps({'python':
                "tool('create_doc', {'path':'proposal.md', 'input':'Cancelled proposal'})"
            })}}],
            'usage': {'prompt_tokens': 1, 'completion_tokens': 1},
        }).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_):
        pass


def main(binary, python):
    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Provider)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    master, slave = pty.openpty()
    process = None
    try:
        with tempfile.TemporaryDirectory() as root:
            env = dict(os.environ, OPENAI_API_KEY='fixture-not-a-real-key',
                       OPENAI_BASE_URL=f'http://127.0.0.1:{server.server_port}/',
                       HICKORY_KEY_STORE=str(Path(root) / 'absent-keys.json'))
            process = subprocess.Popen(
                [binary, '--python', python, '--dir', root, '--provider', 'openai',
                 '--max-turns', '2', 'Propose a new note'],
                stdin=slave, stdout=slave, stderr=slave, env=env,
                start_new_session=True,
            )
            os.close(slave)
            output = b''
            deadline = time.monotonic() + 30
            while b"Type 'approve'" not in output:
                assert time.monotonic() < deadline, 'CLI never reached terminal approval'
                ready, _, _ = select.select([master], [], [], 0.2)
                if ready:
                    output += os.read(master, 65536)
                assert len(output) < 1_048_576, 'Unexpected CLI output growth'
            process.send_signal(signal.SIGINT)
            assert process.wait(timeout=5) != 0, 'Cancellation was reported successful'
            assert not Path(root, 'proposal.md').exists(), 'Cancelled edit was applied'
            records = list(Path(root, 'sessions').glob('*.md'))
            assert records and 'Python agent cancelled' in records[0].read_text()
            print('CLI cancels promptly while waiting for terminal approval; no write occurred')
    finally:
        if process is not None and process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        os.close(master)
        server.shutdown()
        server.server_close()


if __name__ == '__main__':
    main(*sys.argv[1:])
