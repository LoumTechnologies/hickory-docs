# Clients

A generator's first forty lines are always the same: spawn the model server,
frame the JSON, notice that GraphQL reports failure inside a
successful-looking body, and reach into an untyped response. None of it is
specific to a project or a language, and every generator rewrites it.

Measured on the warehouse's API generator, that plumbing was **36% of the
file** and the three rules it existed to serve were **9%**.

| | lines | pays for itself at |
|---|---|---|
| generator, hand-rolled transport | 326 | ~16 endpoints |
| generator, using the client | 202 | ~10 endpoints |
| the client, written once | 189 | — |

## What a client is, and is not

**Is:** the transport, the error check, attribute access over the response,
and a small buffer for building text. Four things, all of them the same in
every language.

**Is not:** a typed client generated from the schema. That would need a code
generator per language — the thing this exists to make unnecessary — and
attribute access gets most of the benefit for none of the machinery. What it
does buy over raw dictionaries is the failure mode: a mistyped field raises
here and returns `None` from a dict, and a generator that silently reads
`None` emits silently wrong code.

## A bug worth keeping

The first version of the Python client made `Node` a fresh wrapper on every
access, so two reads of the same field produced objects that compared
unequal. A generator testing `parameter in route_params` then found nothing
ever matched, and emitted a handler taking `string sku, string sku`. It was
caught by diffing the rewritten generator's output against the previous
generator's, byte for byte — which is the only reason it is not still there,
and the argument for keeping an equivalence gate on anything that generates.

`Node.__eq__` compares the underlying response by identity now, and access is
memoised.

## What is here

| language | client | status |
|---|---|---|
| Python | `python/hick_model.py` | in use by the warehouse's generator |
| TypeScript | — | not written; the same four things |
| Go | — | not written; the same four things |

Python first because a generator can be written in any language and Python is
the one most people reach for when the answer is "not the language I am
generating". The other two are about a hundred and ninety lines each, and the
measurement above is the case for writing them.
