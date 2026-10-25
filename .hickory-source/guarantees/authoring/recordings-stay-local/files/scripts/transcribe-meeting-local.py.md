<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# scripts/transcribe-meeting-local.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Meeting recordings remain recoverable local evidence](../../contract.md)

Captured SHA-256: `d3f1b7fb0f731700229b2a492fde475dede11fbf29346095c6b71dfbff83a39a`.

<book:entry path="scripts/transcribe-meeting-local.py">
#!/usr/bin/env python3
"""Explicit local Faster Whisper adapter. Never downloads a model.

Configure the venv Python as executable and arguments:
["/absolute/path/transcribe-meeting-local.py", "--model", "{model}", "{audio}"]
The model is a preinstalled local CTranslate2 model directory.
"""
import argparse
import os
from pathlib import Path


def timestamp(seconds):
    milliseconds = max(0, round(seconds * 1000))
    hours, milliseconds = divmod(milliseconds, 3_600_000)
    minutes, milliseconds = divmod(milliseconds, 60_000)
    seconds, milliseconds = divmod(milliseconds, 1000)
    return f'{hours:02}:{minutes:02}:{seconds:02}.{milliseconds:03}'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--model', type=Path, required=True)
    parser.add_argument('audio', type=Path)
    args = parser.parse_args()
    if not args.model.is_dir() or not args.audio.is_file():
        parser.error('Choose existing local model and audio paths; no download is performed')
    if not (args.model / 'tokenizer.json').is_file():
        parser.error('A complete local model must include tokenizer.json; refusing a tokenizer download')
    os.environ['HF_HUB_OFFLINE'] = '1'
    os.environ['TRANSFORMERS_OFFLINE'] = '1'
    from faster_whisper import WhisperModel
    model = WhisperModel(str(args.model.resolve()), device='cpu', compute_type='int8', local_files_only=True)
    segments, _ = model.transcribe(str(args.audio.resolve()), beam_size=5)
    print('WEBVTT\n')
    for segment in segments:
        # No speaker ID is invented. Timestamp estimates remain model output.
        text = segment.text.strip().replace('-->', '→').replace('\n', ' ')
        print(f'{timestamp(segment.start)} --> {timestamp(segment.end)}\n{text}\n')


if __name__ == '__main__':
    main()
</book:entry>

</book:doc>
