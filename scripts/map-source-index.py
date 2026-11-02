"""Project a bulk TypeScript index through exact Hickory byte origins."""
import bisect
import hashlib
import json
import re


class Coordinates:
    def __init__(self, text):
        self.text = text
        self.raw = text.encode('utf-8')
        self.utf16_breaks, self.byte_breaks = [0], [0]
        self.invalid_utf16, self.invalid_bytes = set(), set()
        self.lines = [0]
        byte_delta = utf16_delta = 0
        for match in re.finditer(r'\r\n?|\n|[^\x00-\x7f]', text):
            char, start = match.group(), match.start()
            byte, utf16 = start+byte_delta, start+utf16_delta
            width, units = len(char.encode('utf-8')), len(char.encode('utf-16-le'))//2
            if width != units:
                self.invalid_bytes.update(range(byte+1, byte+width))
                self.invalid_utf16.update(range(utf16+1, utf16+units))
                self.utf16_breaks.append(utf16+units)
                self.byte_breaks.append(byte+width)
            byte_delta += width-len(char)
            utf16_delta += units-len(char)
            if char in ['\n', '\r', '\r\n']: self.lines.append(utf16+units)
        self.length = len(text)+utf16_delta

    def byte_at(self, offset):
        if offset < 0 or offset > self.length or offset in self.invalid_utf16: raise KeyError(offset)
        at = bisect.bisect_right(self.utf16_breaks, offset)-1
        return offset + self.byte_breaks[at]-self.utf16_breaks[at]

    def utf16_at(self, offset):
        if offset < 0 or offset > len(self.raw) or offset in self.invalid_bytes: raise KeyError(offset)
        at = bisect.bisect_right(self.byte_breaks, offset)-1
        return offset + self.utf16_breaks[at]-self.byte_breaks[at]

    def position(self, offset):
        line = bisect.bisect_right(self.lines, offset) - 1
        return dict(line=line, character=offset-self.lines[line])


def project_index(index, documents, files):
    if index.get('version') != 1 or index.get('coordinateEncoding') != 'utf-16':
        raise ValueError('Unsupported model index coordinates')
    sources = {item['path']: Coordinates(item['source']) for item in documents}
    generated = {item['path']: item for item in files}
    coordinates = {}
    for file in index['files']:
        entry = generated.get(file['path'])
        if entry is None: raise ValueError('Model indexed an unselected file')
        raw = entry['source'].encode('utf-8') if 'source' in entry else bytes(entry['bytes'])
        if hashlib.sha256(raw).hexdigest() != file['sha256']:
            raise ValueError('Model index is not bound to the constructed bytes')
        coordinates[file['path']] = Coordinates(raw.decode('utf-8'))

    def locate(file, start, end):
        coords = coordinates.get(file)
        if coords is None: return []
        try: left, right = coords.byte_at(start), coords.byte_at(end)
        except KeyError: raise ValueError('Model range splits a UTF-16 surrogate') from None
        if right < left: raise ValueError('Reversed model range')
        found = []
        for origin in generated[file].get('origins', []):
            a, b = origin['output']
            source = sources.get(origin['document'])
            if source is None or left < a or right > b: continue
            begin = origin['source'][0] + left-a
            finish = begin + right-left
            if finish > origin['source'][1] or source.raw[begin:finish] != coords.raw[left:right]:
                raise ValueError('Hickory origin does not preserve indexed bytes')
            try: mapped = (source.utf16_at(begin), source.utf16_at(finish))
            except KeyError: raise ValueError('Hickory origin splits a UTF-8 character') from None
            found.append((origin['document'], *mapped))
        return sorted(set(found))

    def location(span):
        path, start, end = span
        return dict(uri='hick:///'+path, range=dict(start=sources[path].position(start), end=sources[path].position(end)))
    def unique(values):
        return list({json.dumps(value, sort_keys=True): value for value in values}.values())
    def answer(value): return dict(state='captured', result=value)
    symbols = {symbol['id']: symbol for symbol in index['symbols']}
    declarations = {identity: unique([location(span) for declaration in symbol['declarations'] for span in locate(declaration['file'], declaration['from'], declaration['to'])]) for identity, symbol in symbols.items()}
    references = {identity: [] for identity in symbols}
    mapped = []
    call_contexts = {}
    for file in index['files']:
        # AST call ranges nest or are disjoint. Sweep them once, preserving the
        # innermost active call instead of scanning every call per identifier.
        calls = sorted(file['calls'], key=lambda call: (call['from'], -call['to']))
        active, pointer = [], 0
        for occurrence in sorted(file['occurrences'], key=lambda item: (item['from'], item['to'])):
            while pointer < len(calls) and calls[pointer]['from'] <= occurrence['from']:
                call = calls[pointer]
                while active and active[-1]['to'] <= call['from']: active.pop()
                active.append(call)
                pointer += 1
            while active and active[-1]['to'] < occurrence['to']: active.pop()
            if active: call_contexts[(file['path'], occurrence['from'], occurrence['to'])] = active[-1]
    for file in index['files']:
        for occurrence in file['occurrences']:
            targets = locate(file['path'], occurrence['from'], occurrence['to'])
            for identity in occurrence['symbols']:
                if identity not in symbols: raise ValueError('Occurrence names an unknown symbol')
                references[identity].extend(dict(location(target), kind=3 if occurrence['isWrite'] else 2) for target in targets)
            mapped.append((file, occurrence, targets))
    references = {identity: unique(values) for identity, values in references.items()}
    pages, occurrences, semantics = {}, {}, {}
    symbol_kinds = {'class': 5, 'interface': 11, 'enum': 10, 'type': 5, 'typeParameter': 26, 'method': 6, 'property': 7, 'function': 12, 'namespace': 3}
    token_types = ['class', 'enum', 'interface', 'namespace', 'typeParameter', 'type', 'parameter', 'variable', 'enumMember', 'property', 'function', 'method']
    for file, item, targets in mapped:
        for target in targets:
            path, start, end = target
            page = pages.setdefault(path, dict(origin='code-model:typescript', provider_count=1,
                capabilities=dict(hoverProvider=True, definitionProvider=True, referencesProvider=True,
                    documentHighlightProvider=True, documentSymbolProvider=True, foldingRangeProvider=True, inlayHintProvider=True,
                    semanticTokensProvider=dict(legend=dict(tokenTypes=token_types, tokenModifiers=['declaration', 'static', 'async', 'readonly', 'defaultLibrary', 'local']), full=True)),
                queries={}, diagnostics=[], occurrences=[], shared={}))
            key = (path, start, end)
            occurrence = occurrences.get(key)
            if occurrence is None:
                occurrence = dict(from_=start, to=end, position=sources[path].position(start), symbolIds=[],
                    hover=answer(dict(contents=[], range=location(target)['range'])), signatureHelp=answer(dict(signatures=[], activeSignature=0, activeParameter=0)))
                occurrence['from'] = occurrence.pop('from_')
                occurrences[key] = occurrence
                page['occurrences'].append(occurrence)
            occurrence['symbolIds'] = sorted(set(occurrence['symbolIds'] + item['symbols']))
            coords = coordinates[file['path']]
            name = coords.raw[coords.byte_at(item['from']):coords.byte_at(item['to'])].decode('utf-8')
            text = f"{name}: {item['type']}"
            if item['unresolved']: text += '\n// Binding unresolved; see diagnostics.'
            contents = [dict(language='typescript', value=text), dict(language='text', value='Context: '+file['path'])]
            contents.extend(symbols[identity]['documentation'] for identity in item['symbols'] if symbols[identity]['documentation'])
            occurrence['hover']['result']['contents'] = unique(occurrence['hover']['result']['contents'] + contents)
            call = call_contexts.get((file['path'], item['from'], item['to']))
            if call:
                help = occurrence['signatureHelp']['result']['signatures']
                occurrence['signatureHelp']['result']['signatures'] = unique(help + [dict(label=call['signature'])])
            for identity in item['symbols']:
                page['shared'][identity] = dict(definition=answer(declarations[identity]), references=answer(references[identity]))
            if item['isDefinition']:
                queries = page['queries']
                queries.setdefault('documentSymbol', answer([]))['result'].append(dict(name=name, kind=symbol_kinds.get(next((symbols[identity].get('kind') for identity in item['symbols']), 'variable'), 13), range=location(target)['range'], selectionRange=location(target)['range']))
    for file in index['files']:
        for start, length, classification in zip(file['classifications'][::3], file['classifications'][1::3], file['classifications'][2::3]):
            token_type = (classification >> 8)-1
            if not 0 <= token_type < len(token_types): continue
            for path, left, right in locate(file['path'], start, start+length):
                if path not in pages: continue
                position = sources[path].position(left)
                if sources[path].position(right)['line'] != position['line']: continue
                semantics.setdefault(path, set()).add((position['line'], position['character'], right-left, token_type, classification & 255))
        for fold in file['folds']:
            for path, left, right in locate(file['path'], fold['from'], fold['to']):
                if path in pages:
                    pages[path]['queries'].setdefault('foldingRange', answer([]))['result'].append(dict(startLine=sources[path].position(left)['line'], startCharacter=sources[path].position(left)['character'], endLine=sources[path].position(right)['line'], endCharacter=sources[path].position(right)['character']))
        for hint in file['hints']:
            for path, left, right in locate(file['path'], hint['position'], hint['position']):
                if path in pages:
                    pages[path]['queries'].setdefault('inlayHint', answer([]))['result'].append(dict(position=sources[path].position(left), label=hint['text'], kind=2 if hint['kind']=='Parameter' else 1, paddingLeft=hint['whitespaceBefore'], paddingRight=hint['whitespaceAfter']))
    for diagnostic in index['diagnostics']:
        for path, left, right in locate(diagnostic['file'], diagnostic['from'], diagnostic['to']):
            if path in pages:
                pages[path]['diagnostics'].append(dict(range=dict(start=sources[path].position(left), end=sources[path].position(right)), message=diagnostic['message'], code=diagnostic['code'], severity={0: 2, 1: 1, 2: 4, 3: 3}.get(diagnostic['category'], 3), source='typescript-code-model'))
    for path, page in pages.items():
        data, previous_line, previous_character = [], 0, 0
        for line, character, length, kind, modifiers in sorted(semantics.get(path, [])):
            data.extend([line-previous_line, character-previous_character if line==previous_line else character, length, kind, modifiers])
            previous_line, previous_character = line, character
        page['queries']['semanticTokens/full'] = answer(dict(data=data))
        for method in ['documentSymbol', 'foldingRange', 'inlayHint']: page['queries'].setdefault(method, answer([]))
        page['occurrences'].sort(key=lambda item: (item['from'], item['to']))
    return pages
