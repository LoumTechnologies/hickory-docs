"""Explicit local inference transport for the independent agent and focused experiments."""
from dataclasses import dataclass, asdict
import json
import time
import urllib.request
from urllib.parse import urlparse
from .core import ModelResponse

@dataclass(frozen=True)
class LocalSettings:
    context_tokens: int = 8192
    output_tokens: int = 768
    temperature: float = 0.2
    top_p: float = 0.95
    top_k: int = 20
    threads: int = 6
    think: bool = False
    seed: int = 42
    response_format: str = 'json'
    timeout_seconds: int = 900

    def validate(self):
        if not 1024 <= self.context_tokens <= 131072:
            raise ValueError('context_tokens must be between 1024 and 131072')
        if not 1 <= self.output_tokens < self.context_tokens:
            raise ValueError('output_tokens must leave room for input')
        if not 0 <= self.temperature <= 2 or not 0 < self.top_p <= 1:
            raise ValueError('Invalid sampling settings')
        if not 1 <= self.top_k <= 1000 or not 1 <= self.threads <= 256:
            raise ValueError('Invalid top_k or thread count')
        if not 1 <= self.timeout_seconds <= 14400:
            raise ValueError('timeout_seconds must be between 1 and 14400')
        if self.response_format not in ('json', 'text'):
            raise ValueError('response_format must be json or text')
        return self

class OllamaModel:
    """Native local requests: explicit allocation, CPU execution and measured phases.

    No implicit model download, cloud fallback, credential lookup or shell command.
    The input estimate is a planning heuristic, not a tokenizer or proof of coverage.
    """
    def __init__(self, model, settings=None, base_url='http://127.0.0.1:11434'):
        parsed = urlparse(base_url)
        if parsed.scheme != 'http' or parsed.hostname not in ('127.0.0.1', 'localhost', '::1') or parsed.username:
            raise ValueError('Local experiments require a loopback HTTP Ollama endpoint')
        self.model, self.url = model, base_url.rstrip('/')
        self.settings = (settings or LocalSettings()).validate()
        self.calls = []
        self.identity = self._request('/api/show', {'model': model})
        self.identity['ollama_version'] = self._request('/api/version').get('version')
        installed = self._request('/api/tags').get('models', [])
        selected = next((item for item in installed if item.get('name') == model), {})
        self.identity['installed_digest'] = selected.get('digest')
        self.identity['installed_size'] = selected.get('size')
        # Refuse loading a larger model into an already busy Linux host. Existing
        # residency is credited only for this model, never for another process.
        from pathlib import Path
        memory = Path('/proc/meminfo')
        if memory.exists() and selected.get('size'):
            available = int(next(line.split()[1] for line in memory.read_text().splitlines()
                                 if line.startswith('MemAvailable:'))) * 1024
            resident = sum(item.get('size', 0) for item in self._request('/api/ps').get('models', [])
                           if item.get('name') == model)
            if available + resident < selected['size'] + 1_500_000_000:
                raise RuntimeError('Insufficient available memory for this model plus context headroom')
        if self.identity.get('remote_host') or model.endswith(':cloud'):
            raise ValueError('Cloud-backed models are not local experiments')

    def _request(self, path, body=None):
        request = urllib.request.Request(self.url + path,
            data=json.dumps(body).encode() if body is not None else None, headers={'Content-Type': 'application/json'})
        class NoRedirect(urllib.request.HTTPRedirectHandler):
            def redirect_request(self, *args, **kwargs):
                raise ValueError('Local inference does not follow redirects')
        opener = urllib.request.build_opener(NoRedirect)
        with opener.open(request, timeout=self.settings.timeout_seconds) as response:
            raw = response.read(4_194_305)
        if len(raw) > 4_194_304:
            raise ValueError('Ollama response exceeds 4 MiB')
        return json.loads(raw)

    def complete(self, messages):
        settings = self.settings
        estimated = sum((len(m['content'].encode()) + 2) // 3 + 8 for m in messages)
        if estimated + settings.output_tokens > settings.context_tokens:
            raise RuntimeError('Estimated context budget exhausted; select or condense documents before inference')
        started = time.monotonic()
        body = {'model': self.model, 'messages': messages,
            'stream': False, 'think': settings.think, 'keep_alive': '30m',
            'options': {'num_ctx': settings.context_tokens, 'num_predict': settings.output_tokens,
                'num_thread': settings.threads, 'num_gpu': 0, 'temperature': settings.temperature,
                'top_p': settings.top_p, 'top_k': settings.top_k, 'seed': settings.seed}}
        if settings.response_format == 'json':
            body['format'] = 'json'
        result = self._request('/api/chat', body)
        elapsed = time.monotonic() - started
        metrics = {key: result.get(key) for key in ('load_duration', 'prompt_eval_count',
            'prompt_eval_duration', 'eval_count', 'eval_duration', 'total_duration', 'done_reason')}
        metrics.update(elapsed_seconds=elapsed, estimated_input_tokens=estimated,
                       settings=asdict(settings), model=self.model)
        self.calls.append(metrics)
        # An interrupted reasoning trace is never an action fallback.
        return ModelResponse(result.get('message', {}).get('content', ''),
                             result.get('done_reason'), metrics)


