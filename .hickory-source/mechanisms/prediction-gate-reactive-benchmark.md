<reactive:doc xmlns:reactive="http://www.hickorydocs.com/1.0">
# Repair reactive algorithms against long event traces

For maintainers reproducing the user-requested complex benchmark. The
[protocol and provenance](../../experiments/prediction-gate/reactive-plan.md)
explain the six tasks, larger shared budgets and native-runtime exclusions.
The [existing harness owner](prediction-gate-experiment.md) supplies confinement,
production gating and trial retention. This chapter adds semantic extractions;
it does not repair product code or renew existing explanation gaps.

## Keep generation and oracle code evaluator-only

Only the selected starting kernel, public inputs and repair contract enter model context. Randomness creates frozen test traces from named seeds, not adaptive tasks.

<reactive:copy id="prediction-reactive-prelude">
"""Frozen reactive-v4 fixtures. Evaluator-only; never expose solutions or held-out cases."""
import random

</reactive:copy>

## Separate membership, readiness and retained values

Completion can stop a source from blocking while retaining its cached value. A poll drains queued membership changes before deciding quiescence, and cancellation has priority over queued data. This model does not exercise a Tokio waker.

<reactive:copy id="prediction-reactive-dcl-kernel">
DCL = '''def combine(state):
    return [state['latest'][key] for key in sorted(state['latest'])]

def ready(state):
    return bool(state['active']) and not state['unready']

def apply_message(state, message):
    kind, key = message[:2]
    if kind == 'add':
        if key not in state['seen']:
            state['seen'].append(key)
            state['active'].append(key)
            state['unready'].append(key)
            state['ever'] = True
    elif kind == 'remove':
        if key in state['active']:
            state['active'].remove(key)
        if key in state['unready']:
            state['unready'].remove(key)
        state['latest'].pop(key, None)
    elif kind == 'complete':
        if key in state['active']:
            state['active'].remove(key)
        if key in state['unready']:
            state['unready'].remove(key)
    elif kind == 'data' and key in state['active']:
        state['latest'][key] = message[2]
        if key in state['unready']:
            state['unready'].remove(key)
        if ready(state):
            return ['emit', combine(state)]
    return None

def poll(state, quiescent):
    if state['ended'] or state['cancelled']:
        state['ended'] = True
        return ['done']
    while state['queue']:
        result = apply_message(state, state['queue'].pop(0))
        if result is not None:
            return result
    if quiescent and state['ever'] and not state['active']:
        state['ended'] = True
        return ['done']
    return ['pending']

def run(actions, quiescent):
    state = {'active':[], 'unready':[], 'seen':[], 'latest':{},
             'queue':[], 'ever':False, 'cancelled':False, 'ended':False}
    observations = []
    for action in actions:
        if action[0] == 'enqueue':
            state['queue'].append(action[1])
        elif action[0] == 'shutdown':
            state['cancelled'] = True
        elif action[0] == 'poll':
            observations.append(poll(state, quiescent))
    return {'polls':observations, 'active':sorted(state['active']),
            'unready':sorted(state['unready']), 'latest':[[k,state['latest'][k]] for k in sorted(state['latest'])],
            'remaining':len(state['queue']), 'ended':state['ended']}
'''

</reactive:copy>

## Recompute offsets for each alias callback

An outer occurrence owns its own delivered buffer. Shared inner streams notify subscriptions in creation order, so a callback must use the current preceding buffers, even after outer insertion reverses their order.

<reactive:copy id="prediction-reactive-flatten-kernel">
FLATTEN = '''def offset(slots, index):
    return sum(len(slot['values']) for slot in slots[:index])

def publish(result, changes, step, kind, index, values):
    if not values:
        return
    if kind == 'add':
        result[index:index] = values
    else:
        del result[index:index + len(values)]
    changes.append([step, kind, index, list(values)])

def subscription_order(slot):
    return slot['serial']

def inner_change(slots, result, changes, step, source, index, values, remove):
    # Subscription delivery follows creation order, not current outer position.
    for slot in sorted(slots, key=subscription_order):
        if slot['source'] != source:
            continue
        position = slots.index(slot)
        base = offset(slots, position)
        if remove:
            old = slot['values'][index:index + values]
            publish(result, changes, step, 'remove', base + index, old)
            del slot['values'][index:index + values]
        else:
            publish(result, changes, step, 'add', base + index, values)
            slot['values'][index:index] = values

def run(actions):
    sources = {}
    slots = []
    result = []
    changes = []
    snapshots = []
    serial = 0
    disposed = False
    for step, action in enumerate(actions):
        kind = action[0]
        if kind == 'source':
            sources[action[1]] = list(action[2])
        elif kind == 'insert':
            name, index, values = action[1:]
            sources[name][index:index] = values
            if not disposed:
                inner_change(slots, result, changes, step, name, index, values, False)
        elif kind == 'delete':
            name, index, count = action[1:]
            del sources[name][index:index + count]
            if not disposed:
                inner_change(slots, result, changes, step, name, index, count, True)
        elif kind == 'attach' and not disposed:
            index, token, source = action[1:]
            slot = {'token':token, 'source':source, 'values':list(sources[source]), 'serial':serial}
            serial += 1
            base = offset(slots, index)
            slots.insert(index, slot)
            publish(result, changes, step, 'add', base, slot['values'])
        elif kind == 'detach' and not disposed:
            index, count = action[1:]
            base = offset(slots, index)
            old = [value for slot in slots[index:index+count] for value in slot['values']]
            publish(result, changes, step, 'remove', base, old)
            del slots[index:index+count]
        elif kind == 'dispose':
            disposed = True
            slots.clear()
        snapshots.append(list(result))
    return {'changes':changes, 'snapshots':snapshots,
            'subscriptions':[[name,sum(slot['source']==name for slot in slots)] for name in sorted(sources)]}
'''

</reactive:copy>

## Retain group identity after the last member leaves

Published group handles survive empty membership. Value changes remove the old payload before publishing the new one, including updates that stay in the same key.

<reactive:copy id="prediction-reactive-groups-kernel">
GROUPS = '''def group_key(value, modulus):
    return abs(value) % modulus

def remove_member(groups, members, token, changes, step):
    if token not in members:
        return
    key, value = members.pop(token)
    del groups[key]['members'][token]
    changes.append([step,'remove',key,token,value])
    # Published groups retain identity even when their membership becomes empty.

def add_member(groups, members, token, value, modulus, changes, step):
    key = group_key(value, modulus)
    if key not in groups:
        groups[key] = {'birth':len(groups)+1, 'members':{}}
        changes.append([step,'group',key,groups[key]['birth']])
    groups[key]['members'][token] = value
    members[token] = [key,value]
    changes.append([step,'add',key,token,value])

def snapshot(groups):
    return [[key, groups[key]['birth'], [[token, value] for token,value in sorted(groups[key]['members'].items())]] for key in sorted(groups)]

def run(actions, modulus):
    groups = {}
    members = {}
    changes = []
    snapshots = []
    disposed = False
    for step, action in enumerate(actions):
        kind, token = action[:2]
        if not disposed:
            if kind == 'add' and token not in members:
                add_member(groups,members,token,action[2],modulus,changes,step)
            elif kind == 'remove':
                remove_member(groups,members,token,changes,step)
            elif kind == 'value' and token in members and members[token][1] != action[2]:
                remove_member(groups,members,token,changes,step)
                add_member(groups,members,token,action[2],modulus,changes,step)
            elif kind == 'dispose':
                disposed = True
        snapshots.append(snapshot(groups))
    return {'changes':changes,'snapshots':snapshots,'subscribed':[] if disposed else sorted(members)}
'''

</reactive:copy>

## Count owners rather than distinct source names

Aliases each own a contribution. A shared value disappears only when the last contribution leaves; disposing freezes the published view while unsubscribing from future updates.

<reactive:copy id="prediction-reactive-union-kernel">
UNION = '''def adjust(counts, changes, step, value, amount):
    old = counts.get(value,0)
    new = old + amount
    if new:
        counts[value] = new
    else:
        counts.pop(value,None)
    if old == 0 and new > 0:
        changes.append([step,'add',value])
    elif old > 0 and new == 0:
        changes.append([step,'remove',value])

def run(actions):
    sources = {}
    subscriptions = {}
    counts = {}
    changes = []
    snapshots = []
    disposed = False
    for step,action in enumerate(actions):
        kind = action[0]
        if kind == 'source':
            sources[action[1]] = set(action[2])
        elif kind == 'attach' and not disposed:
            token, source = action[1:]
            if token not in subscriptions:
                subscriptions[token] = source
                for value in sorted(sources[source]):
                    adjust(counts,changes,step,value,1)
        elif kind == 'detach' and not disposed:
            source = subscriptions.pop(action[1],None)
            if source is not None:
                for value in sorted(sources[source]):
                    adjust(counts,changes,step,value,-1)
        elif kind in ['insert','delete']:
            source,value = action[1:]
            present = value in sources[source]
            if kind == 'insert':
                sources[source].add(value)
            else:
                sources[source].discard(value)
            if present != (kind == 'insert') and not disposed:
                amount = sum(name == source for name in subscriptions.values())
                if amount:
                    adjust(counts,changes,step,value,amount if kind=='insert' else -amount)
        elif kind == 'dispose':
            disposed = True
            subscriptions.clear()
        snapshots.append(sorted(counts))
    return {'changes':changes,'snapshots':snapshots,'subscribed':sorted(subscriptions)}
'''

</reactive:copy>

## Propagate index shifts through a composed predicate

Insertions and deletions change every later source index. Observable edits remove vanished identities right-to-left before adding newly visible identities left-to-right, preserving surviving identities.

<reactive:copy id="prediction-reactive-where-kernel">
WHERE = '''def included(value, index, modulus):
    return (value + index) % modulus == 0

def visible(rows, modulus):
    return [[token,value*2] for index,(token,value) in enumerate(rows) if included(value,index,modulus)]

def difference(old, new, changes, step):
    # Canonical identity-preserving edit script: removals right-to-left, additions left-to-right.
    new_ids = [row[0] for row in new]
    current = [list(row) for row in old]
    for index in range(len(current)-1,-1,-1):
        if current[index][0] not in new_ids:
            changes.append([step,'remove',index,current[index]])
            current.pop(index)
    for index,row in enumerate(new):
        if index == len(current) or current[index][0] != row[0]:
            changes.append([step,'add',index,row])
            current.insert(index,list(row))
        elif current[index] != row:
            changes.append([step,'replace',index,row])
            current[index] = list(row)

def run(actions, modulus):
    rows = []
    view = []
    changes = []
    snapshots = []
    indices = []
    disposed = False
    for step,action in enumerate(actions):
        if not disposed:
            if action[0] == 'insert':
                rows[action[1]:action[1]] = [list(row) for row in action[2]]
            elif action[0] == 'delete':
                del rows[action[1]:action[1]+action[2]]
            elif action[0] == 'value':
                for row in rows:
                    if row[0] == action[1]:
                        row[1] = action[2]
            elif action[0] == 'dispose':
                disposed = True
            if not disposed:
                new = visible(rows,modulus)
                difference(view,new,changes,step)
                view = new
        snapshots.append([list(row) for row in view])
        indices.append([] if disposed else [[row[0],index] for index,row in enumerate(rows)])
    return {'changes':changes,'snapshots':snapshots,'indices':indices}
'''

</reactive:copy>

## Compute DCL expectations with a separate state representation

The oracle derives readiness from source records and advances an inbox cursor. It does not invoke the candidate poll, readiness or combining functions.

<reactive:copy id="prediction-reactive-dcl-oracle">
# Evaluator-only references use recomputation, rather than candidate incremental helpers.
def dcl_oracle(actions, quiescent):
    records, seen, inbox, cursor = {}, set(), [], 0
    ever = cancelled = ended = False
    results = []
    for action in actions:
        if action[0] == 'enqueue':
            inbox.append(action[1]); continue
        if action[0] == 'shutdown':
            cancelled = True; continue
        if ended or cancelled:
            ended = True; results.append(['done']); continue
        emitted = False
        while cursor < len(inbox):
            message = inbox[cursor]; cursor += 1
            op, key = message[:2]
            if op == 'add' and key not in seen:
                seen.add(key); records[key] = [True,False,None]; ever = True
            elif op == 'remove':
                records.pop(key,None)
            elif op == 'complete' and key in records:
                records[key][0] = False
            elif op == 'data' and key in records and records[key][0]:
                records[key][1:] = [True,message[2]]
                running = [r for r in records.values() if r[0]]
                if running and all(r[1] for r in running):
                    results.append(['emit',[records[k][2] for k in sorted(records) if records[k][1]]])
                    emitted = True; break
        if not emitted:
            ended = quiescent and ever and not any(r[0] for r in records.values())
            results.append(['done'] if ended else ['pending'])
    return {'polls':results,'active':sorted(k for k,r in records.items() if r[0]),
            'unready':sorted(k for k,r in records.items() if r[0] and not r[1]),
            'latest':[[k,records[k][2]] for k in sorted(records) if records[k][1]],
            'remaining':len(inbox)-cursor,'ended':ended}

</reactive:copy>

## Recompute the reference flattened view

Reference occurrences reconstruct the flat view from their delivered buffers. The independent anchor test below fixes callback offsets when creation and outer orders disagree.

<reactive:copy id="prediction-reactive-flatten-oracle">
def flatten_oracle(actions):
    sources, occurrences, events, snapshots = {}, [], [], []
    serial = 0; frozen = None
    for step,a in enumerate(actions):
        op = a[0]
        flatten = lambda: [v for slot in occurrences for v in slot[3]]
        if op == 'source':
            sources[a[1]] = list(a[2])
        elif op in ['insert','delete']:
            name,index,data = a[1:]
            if op == 'insert': sources[name][index:index] = data
            else: del sources[name][index:index+data]
            if frozen is None:
                for slot in sorted(occurrences,key=lambda x:x[0]):
                    if slot[2] != name: continue
                    pos = sum(len(o[3]) for o in occurrences[:occurrences.index(slot)])+index
                    if op == 'insert':
                        values = list(data); slot[3][index:index] = values
                    else:
                        values = slot[3][index:index+data];del slot[3][index:index+data]
                    if values: events.append([step,'add' if op=='insert' else 'remove',pos,values])
        elif frozen is None and op == 'attach':
            index,token,name=a[1:]
            before=sum(len(o[3]) for o in occurrences[:index]); values=list(sources[name])
            occurrences.insert(index,[serial,token,name,values]);serial+=1
            if values:events.append([step,'add',before,list(values)])
        elif frozen is None and op == 'detach':
            index,count=a[1:];before=sum(len(o[3]) for o in occurrences[:index])
            values=[v for o in occurrences[index:index+count] for v in o[3]]
            del occurrences[index:index+count]
            if values:events.append([step,'remove',before,values])
        elif op == 'dispose' and frozen is None:
            frozen=flatten();occurrences=[]
        snapshots.append(list(frozen) if frozen is not None else flatten())
    return {'changes':events,'snapshots':snapshots,'subscriptions':[[k,sum(o[2]==k for o in occurrences)] for k in sorted(sources)]}

</reactive:copy>

## Derive group membership from source identities

The reference stores only current source items and persistent group births, then reconstructs each group. This checks the candidate incremental membership dictionaries without reusing them.

<reactive:copy id="prediction-reactive-groups-oracle">
def groups_oracle(actions, modulus):
    items, births, events, snapshots = {}, {}, [], []
    disposed=False
    for step,a in enumerate(actions):
        op,token=a[:2]
        if not disposed:
            if op=='dispose':disposed=True
            elif op=='remove' and token in items:
                old=items.pop(token);events.append([step,'remove',abs(old)%modulus,token,old])
            elif (op=='add' and token not in items) or (op=='value' and token in items and items[token]!=a[2]):
                if token in items:
                    old=items[token];events.append([step,'remove',abs(old)%modulus,token,old])
                value=a[2];key=abs(value)%modulus
                if key not in births:
                    births[key]=len(births)+1;events.append([step,'group',key,births[key]])
                items[token]=value;events.append([step,'add',key,token,value])
        snapshots.append([[key,births[key],[[t,v] for t,v in sorted(items.items()) if abs(v)%modulus==key]] for key in sorted(births)])
    return {'changes':events,'snapshots':snapshots,'subscribed':[] if disposed else sorted(items)}

</reactive:copy>

## Recompute the reference union from current owners

Set union determines visibility without the candidate reference-count algorithm. Delta expectations follow changes in that visibility, while disposal retains the old publication.

<reactive:copy id="prediction-reactive-union-oracle">
def union_oracle(actions):
    sources, owners, visible, events, snapshots = {}, {}, set(), [], []
    disposed=False
    for step,a in enumerate(actions):
        op=a[0]
        if op=='source':sources[a[1]]=set(a[2])
        elif op in ['insert','delete']:
            if op=='insert':sources[a[1]].add(a[2])
            else:sources[a[1]].discard(a[2])
        elif not disposed and op=='attach' and a[1] not in owners:owners[a[1]]=a[2]
        elif not disposed and op=='detach':owners.pop(a[1],None)
        elif op=='dispose':disposed=True;owners.clear()
        if not disposed:
            new=set().union(*(sources[name] for name in owners.values())) if owners else set()
            events += [[step,'remove',v] for v in sorted(visible-new)]
            events += [[step,'add',v] for v in sorted(new-visible)]
            visible=new
        snapshots.append(sorted(visible))
    return {'changes':events,'snapshots':snapshots,'subscribed':sorted(owners)}

</reactive:copy>

## Recompute index-sensitive visibility and canonical edits

The oracle uses immutable source rows to derive the projected result, then checks the specified edit ordering against identity survivors. It does not call the candidate predicate or diff helper.

<reactive:copy id="prediction-reactive-where-oracle">
def where_oracle(actions, modulus):
    rows, result, events, snapshots, indices = [], [], [], [], []
    disposed=False
    for step,a in enumerate(actions):
        if not disposed:
            op=a[0]
            if op=='insert':rows=rows[:a[1]]+[tuple(r) for r in a[2]]+rows[a[1]:]
            elif op=='delete':rows=rows[:a[1]]+rows[a[1]+a[2]:]
            elif op=='value':rows=[(t,a[2] if t==a[1] else v) for t,v in rows]
            elif op=='dispose':disposed=True
            if not disposed:
                new=[[t,v*2] for i,(t,v) in enumerate(rows) if (v+i)%modulus==0]
                survivors=[r for r in result if r[0] in {n[0] for n in new}]
                events += [[step,'remove',i,r] for i,r in reversed(list(enumerate(result))) if r[0] not in {n[0] for n in new}]
                j=0
                for i,r in enumerate(new):
                    if j==len(survivors) or survivors[j][0]!=r[0]:events.append([step,'add',i,r])
                    else:
                        if survivors[j]!=r:events.append([step,'replace',i,r])
                        j+=1
                result=new
        snapshots.append([list(r) for r in result])
        indices.append([] if disposed else [[t,i] for i,(t,v) in enumerate(rows)])
    return {'changes':events,'snapshots':snapshots,'indices':indices}

</reactive:copy>

## Generate finite mailbox histories

Fixed seeds interleave add, data, remove, complete and poll, including stale IDs, pending work and terminal states. Both completion modes appear, and some histories finish with shutdown and queued data.

<reactive:copy id="prediction-reactive-dcl-traces">
def dcl_trace(seed, length=60):
    rng = random.Random(seed)
    trace=[['poll']]
    for key in [7,2,11]: trace.append(['enqueue',['add',key]])
    trace += [['enqueue',['data',7,3]],['enqueue',['data',2,5]],['poll']]
    for _ in range(length):
        op=rng.choice(['add','data','data','complete','remove','poll','poll'])
        if op=='poll':trace.append(['poll'])
        else:
            key=rng.choice([2,7,11,19,23])
            trace.append(['enqueue',[op,key,rng.randrange(-5,10)] if op=='data' else [op,key]])
    trace += [['poll'] for _ in range(8)]
    if seed%2:trace += [['shutdown'],['enqueue',['data',7,99]],['poll'],['poll']]
    return [trace, bool(seed%2)]

</reactive:copy>

## Keep range-edit inputs valid while stressing aliases

The generator tracks source lengths and outer positions solely to create legal edits. It does not inspect a model answer or choose a trace based on which condition performs better.

<reactive:copy id="prediction-reactive-flatten-traces">
def flatten_trace(seed, length=55):
    rng=random.Random(seed);sources={'a':[1,2],'b':[],'c':[8]};slots=[];serial=0
    trace=[['source',k,list(v)] for k,v in sources.items()]
    for _ in range(length):
        op=rng.choice(['attach','attach','detach','insert','delete'])
        name=rng.choice(list(sources))
        if op=='attach':
            index=rng.randrange(len(slots)+1);token='o'+str(serial);serial+=1
            trace.append(['attach',index,token,name]);slots.insert(index,token)
        elif op=='detach' and slots:
            index=rng.randrange(len(slots));count=rng.randrange(1,len(slots)-index+1)
            trace.append(['detach',index,count]);del slots[index:index+count]
        elif op=='insert':
            index=rng.randrange(len(sources[name])+1);values=[rng.randrange(0,10) for _ in range(rng.randrange(0,4))]
            trace.append(['insert',name,index,values]);sources[name][index:index]=values
        elif op=='delete' and sources[name]:
            index=rng.randrange(len(sources[name]));count=rng.randrange(len(sources[name])-index+1)
            trace.append(['delete',name,index,count]);del sources[name][index:index+count]
    trace += [['dispose'],['insert','a',0,[99]],['detach',0,1]]
    return [trace]

</reactive:copy>

## Exercise repeated identities and key changes

Fixed traces repeatedly remove, update and re-add identities. Odd seeds end in disposal followed by stale notifications, preserving that failure boundary in held-out data.

<reactive:copy id="prediction-reactive-groups-traces">
def groups_trace(seed, length=55):
    rng=random.Random(seed)
    trace=[['add','a',2],['add','b',5],['value','a',8],['remove','b'],['add','c',5]]
    for _ in range(length):
        op=rng.choice(['add','add','remove','value']);token=rng.choice(['a','b','c','d','e'])
        trace.append([op,token] if op=='remove' else [op,token,rng.randrange(-10,11)])
    if seed%2:trace += [['dispose',None],['add','late',9],['value','a',42]]
    return [trace,3 if seed%2 else 4]

</reactive:copy>

## Exercise overlapping owners and reattachment

Small value domains create collisions across sources and aliases. Duplicate operations, detachment and final disposal make first-owner/last-owner errors observable.

<reactive:copy id="prediction-reactive-union-traces">
def union_trace(seed, length=55):
    rng=random.Random(seed)
    trace=[['source','a',[1,2]],['source','b',[2,3]],['source','c',[]]]
    for _ in range(length):
        op=rng.choice(['attach','detach','insert','delete']);source=rng.choice(['a','b','c'])
        if op=='attach':trace.append([op,rng.choice(['x','y','z','w']),source])
        elif op=='detach':trace.append([op,rng.choice(['x','y','z','w'])])
        else:trace.append([op,source,rng.randrange(0,5)])
    trace += [['dispose'],['insert','a',99],['detach','x']]
    return [trace]

</reactive:copy>

## Exercise long index-shift histories

Unique IDs distinguish equal payloads. Range edits shift later indices, value updates can target removed IDs, and disposal suppresses later mutations.

<reactive:copy id="prediction-reactive-where-traces">
def where_trace(seed, length=45):
    rng=random.Random(seed);rows=[];serial=0;trace=[]
    for _ in range(length):
        op=rng.choice(['insert','insert','delete','value'])
        if op=='insert':
            index=rng.randrange(len(rows)+1);new=[]
            for _ in range(rng.randrange(1,4)):
                new.append(['p'+str(serial),rng.randrange(-3,8)]);serial+=1
            trace.append([op,index,new]);rows[index:index]=[r.copy() for r in new]
        elif op=='delete' and rows:
            index=rng.randrange(len(rows));count=rng.randrange(1,len(rows)-index+1)
            trace.append([op,index,count]);del rows[index:index+count]
        elif op=='value':
            token=rng.choice(rows)[0] if rows and rng.random()<.8 else 'removed'
            trace.append([op,token,rng.randrange(-3,8)])
    trace += [['dispose'],['insert',0,[['late',12]]],['value','p0',99]]
    return [trace,3]

</reactive:copy>

## Make the extracted contracts explicit

Source URLs pin inspected LiveLinq code. The DCL reference is local. These task contracts disclose extensions and abstractions rather than claiming native implementations already satisfy every clause.

<reactive:copy id="prediction-reactive-contracts">
LIVE_REVISION = 'a50ba1976b78b0edf635a96074d1895812141564'
LIVE_BASE = 'https://github.com/nloum/LiveLinq/blob/' + LIVE_REVISION + '/'

DCL_REQUIREMENT = '''Repair the deterministic DynamicCombineLatest mailbox model. Inputs contain enqueue(message), poll, and shutdown actions. Messages are add(id), data(id,value), complete(id), remove(id). Each poll drains the FIFO mailbox until exactly one data-triggered emission or until no messages remain. New source IDs are unique and replayed add messages are idempotent. Every live source blocks emission until its first data; cold completed sources stop blocking. Completed sources retain their last values for later combinations until removed. Data from removed/completed/unknown sources is ignored. Membership changes never emit by themselves. Combined snapshots use ascending numeric source ID, including retained completed values. Removing a source discards its retained value and readiness state. Quiescent mode terminates only after draining queued work with no live sources, once any source has existed; an add already queued behind the last completion must be processed. Initially empty is pending. Shutdown terminates before consuming queued work. Terminal streams remain terminal. Return polls plus active/unready/latest/remaining/ended state exactly as the starting API does. This is a deterministic semantic extraction of hick-flow, with an explicit repair contract; it is not a native Tokio scheduler test.'''
FLATTEN_REQUIREMENT = '''Repair the SelectMany trace kernel. source(name,values) defines a source once. attach(index,token,name) creates a unique outer occurrence, subscribes and replays the source's current values. Occurrences may alias the same source. insert(name,index,values) and delete(name,index,count) modify the source; its subscribers receive the delta in subscription CREATION order, regardless of current outer position. Each occurrence maintains its own last-delivered buffer; recompute its offset from CURRENT buffers of preceding occurrences for EACH callback. detach(index,count) unsubscribes a contiguous outer range and emits one combined remove using its cached values; empty ranges emit nothing. No late inner changes reach detached occurrences. dispose unsubscribes everything but freezes the last published view; underlying sources may continue changing. Return exact nonempty add/remove events [action_index,kind,flat_index,values], the flat snapshot after EVERY action, and final per-source subscription counts. Inputs use valid current source/outer indices, counts may be zero, tokens are unique, and values may repeat.'''
GROUP_REQUIREMENT = '''Repair the persistent GroupBy trace kernel. Source items have unique identity tokens and integer values; keys are abs(value) % modulus. add(token,value) is idempotent while that token exists, including duplicate adds with another payload. remove(unknown) and value(removed,...) are ignored. value(token,new) atomically removes the prior membership THEN adds the new membership, unless unchanged; same-key payload changes must also propagate. First appearance of a key publishes one group with a monotonically assigned birth identity. Empty groups remain published and retain that birth identity when repopulated. Equal payloads under distinct tokens remain distinct members. dispose stops callbacks and freezes published groups while removing all subscriptions. Return exact group/member events, per-action sorted snapshots [key,birth,sorted [[token,value]...]], and subscribed IDs. This preserves LiveLinq's tested empty-group behavior and extends the extracted kernel with explicit mutable-value and disposal semantics.'''
UNION_REQUIREMENT = '''Repair this set SelectMany/distinct ownership kernel. source(name,values) defines a set. attach(token,name) creates a subscription and contributes that source's CURRENT set; duplicate token attach is ignored. Different tokens may refer to the same source. detach(token) removes exactly that owner's contribution; unknown tokens do nothing. insert/delete mutate the source set idempotently and update every current owner's contribution. A value appears in the view while AT LEAST ONE owner contributes it; emit add only on 0->positive and remove only on positive->0. On multi-value operations emit changed values in ascending order. Removed sources may mutate silently and later be attached again, replaying current state. dispose removes subscriptions but FREEZES the last published view; later events cannot alter it. Return exact events [action_index,kind,value], a sorted snapshot after every action, and sorted subscribed tokens. This is a semantic extraction/extension of LiveLinq set SelectMany, not an assertion that all these contracts are implemented upstream.'''
WHERE_REQUIREMENT = '''Repair this index-sensitive Where -> Select kernel. insert(index,[[id,value],...]) and delete(index,count) mutate an ordered source. IDs are unique. value(id,new) updates an existing ID or ignores stale removed IDs. An item is visible exactly when (value + CURRENT source index) % modulus == 0; its projection is [id,value*2]. ALL index shifts must affect the predicate, including range insertions/deletions. Produce canonical events: remove vanished visible IDs right-to-left using current visible indices, then add newly visible IDs left-to-right at their new visible indices; emit replace for a surviving ID whose projected payload changes. Existing surviving IDs keep relative order. Record per-action projected snapshots and ALL live [id,current_index] subscriptions. dispose freezes the view and removes subscriptions; late operations do nothing. Return the starting API's exact shape. This extracts the composition of LiveLinq index streams, observable predicates, and SelectMany empty/singleton flattening.'''

def task(identifier, correct, start, oracle, public_inputs, heldout_inputs, instruction, references):
    return dict(id=identifier,start=start,solution=correct,instruction=instruction,
                public=[[args,oracle(*args)] for args in public_inputs],
                hidden=[[args,oracle(*args)] for args in heldout_inputs],
                references=references,benchmark_kind='deterministic_semantic_extraction')

</reactive:copy>

## Freeze six fault-injected tasks and separate public/held-out inputs

Each task has two public traces and twelve independently referenced held-out traces. Starting faults are deliberate benchmark mutations; no new model outcomes are used to select seeds or change acceptance.

<reactive:copy id="prediction-reactive-population">
def reactive_tasks():
    # Each family is frozen independently of any subsequent model outcome.
    first_dcl = [[['poll'],['enqueue',['add',9]],['enqueue',['add',2]],
                  ['enqueue',['data',9,4]],['poll'],['enqueue',['data',2,7]],['poll'],
                  ['enqueue',['complete',9]],['enqueue',['data',2,8]],['poll'],
                  ['enqueue',['remove',9]],['enqueue',['data',9,99]],['enqueue',['data',2,3]],['poll']],False]
    second_dcl = [[['enqueue',['add',1]],['poll'],['enqueue',['complete',1]],
                   ['enqueue',['add',3]],['enqueue',['data',3,10]],['poll'],['poll'],
                   ['shutdown'],['enqueue',['data',3,20]],['poll']],True]
    flatten_public = [
        [[['source','a',[1,2]],['source','b',[]],['attach',0,'old','a'],['attach',0,'new','b'],
          ['insert','b',0,[5,6]],['insert','a',1,[3]],['delete','b',0,1],['detach',0,1],['insert','b',0,[9]]]],
        [[['source','a',[4]],['attach',0,'x','a'],['attach',0,'y','a'],['insert','a',1,[7]],
          ['delete','a',0,1],['detach',1,1],['insert','a',0,[8]],['dispose'],['delete','a',0,1]]]]
    groups_public = [
        [[['add','a',2],['add','b',5],['value','a',8],['remove','a'],['remove','b'],['add','c',11],['value','b',4]],3],
        [[['add','x',1],['add','y',1],['add','x',8],['value','y',2],['value','x',7],['remove','x'],['dispose',None],['add','late',4]],3]]
    union_public = [
        [[['source','a',[1]],['source','b',[1,2]],['attach','x','a'],['attach','y','a'],
          ['insert','a',3],['detach','x'],['attach','z','b'],['delete','a',1],['detach','z'],['dispose'],['insert','a',9]]],
        [[['source','a',[]],['source','b',[2]],['attach','x','a'],['attach','y','b'],['insert','a',2],
          ['detach','y'],['delete','a',2],['insert','b',3],['attach','z','b'],['detach','x']]]]
    where_public = [
        [[['insert',0,[['a',0],['b',2],['c',4]]],['insert',0,[['x',1],['y',2]]],
          ['delete',1,2],['value','a',9],['value','c',5],['dispose'],['value','c',99]],3],
        [[['insert',0,[['a',0],['b',1],['c',0],['d',1]]],['delete',0,2],
          ['insert',1,[['e',2],['f',4]]],['value','c',6],['delete',0,3]],2]]
    readiness = DCL.replace("elif kind == 'data' and key in state['active']:","elif kind == 'data':")
    readiness = readiness.replace("    elif kind == 'complete':\n", "    elif kind == 'complete':\n        state['latest'].pop(key, None)\n")
    quiescence = DCL.replace("    while state['queue']:\n", "    while state['queue']:\n        if quiescent and state['ever'] and not state['active']:\n            state['ended'] = True\n            return ['done']\n")
    quiescence = quiescence.replace("if state['ended'] or state['cancelled']:", "if state['ended'] or (state['cancelled'] and not state['queue']):")
    flatten_start = FLATTEN.replace("sum(len(slot['values']) for slot in slots[:index])", "len(slots[:index])")
    flatten_start = flatten_start.replace("for slot in sorted(slots, key=subscription_order):", "for slot in sorted(slots, key=subscription_order)[:1]:")
    group_start = GROUPS.replace("    # Published groups retain identity even when their membership becomes empty.", "    if not groups[key]['members']:\n        groups.pop(key)")
    group_start = group_start.replace("members[token][1] != action[2]", "members[token][0] != group_key(action[2],modulus)")
    union_start = UNION.replace("new = old + amount", "new = old + (1 if amount > 0 else -1)")
    union_start = union_start.replace("            disposed = True\n", "            disposed = True\n            counts.clear()\n")
    where_start = WHERE.replace("included(value,index,modulus)", "included(value,0,modulus)")
    where_start = where_start.replace("range(len(current)-1,-1,-1)", "range(len(current))")
    seeds=range(1100,1112)
    dcl_ref=[{'path':'crates/hick-flow/src/combine_latest.rs','revision':'6662e6566f9df595747248382317291a92fb0400'}]
    return [
        task('dcl-readiness-retention',DCL,readiness,dcl_oracle,[first_dcl,second_dcl],[dcl_trace(s) for s in seeds],DCL_REQUIREMENT,dcl_ref),
        task('dcl-queued-quiescence',DCL,quiescence,dcl_oracle,[second_dcl,first_dcl],[dcl_trace(s+100) for s in seeds],DCL_REQUIREMENT,dcl_ref),
        task('livelinq-flatten-alias-offsets',FLATTEN,flatten_start,flatten_oracle,flatten_public,[flatten_trace(s) for s in seeds],FLATTEN_REQUIREMENT,[LIVE_BASE+'LiveLinq/Extensions.List.SelectMany.cs',LIVE_BASE+'LiveLinq/List/MovingRanges.cs']),
        task('livelinq-persistent-groups',GROUPS,group_start,groups_oracle,groups_public,[groups_trace(s) for s in seeds],GROUP_REQUIREMENT,[LIVE_BASE+'LiveLinq/Extensions.Set.GroupBy.cs',LIVE_BASE+'LiveLinq.Tests/Extensions.Set.GroupBy.Tests.cs']),
        task('livelinq-distinct-ownership',UNION,union_start,union_oracle,union_public,[union_trace(s) for s in seeds],UNION_REQUIREMENT,[LIVE_BASE+'LiveLinq/Extensions.Set.SelectMany.cs']),
        task('livelinq-index-sensitive-filter',WHERE,where_start,where_oracle,where_public,[where_trace(s) for s in seeds],WHERE_REQUIREMENT,[LIVE_BASE+'LiveLinq/Extensions.List.Where.cs',LIVE_BASE+'LiveLinq/Extensions.List.Indexing.cs'])]

TASKS = reactive_tasks()
</reactive:copy>

<reactive:file path="experiments/prediction-gate/reactive_cases.py"><reactive:paste select="#prediction-reactive-prelude"/><reactive:paste select="#prediction-reactive-dcl-kernel"/><reactive:paste select="#prediction-reactive-flatten-kernel"/><reactive:paste select="#prediction-reactive-groups-kernel"/><reactive:paste select="#prediction-reactive-union-kernel"/><reactive:paste select="#prediction-reactive-where-kernel"/><reactive:paste select="#prediction-reactive-dcl-oracle"/><reactive:paste select="#prediction-reactive-flatten-oracle"/><reactive:paste select="#prediction-reactive-groups-oracle"/><reactive:paste select="#prediction-reactive-union-oracle"/><reactive:paste select="#prediction-reactive-where-oracle"/><reactive:paste select="#prediction-reactive-dcl-traces"/><reactive:paste select="#prediction-reactive-flatten-traces"/><reactive:paste select="#prediction-reactive-groups-traces"/><reactive:paste select="#prediction-reactive-union-traces"/><reactive:paste select="#prediction-reactive-where-traces"/><reactive:paste select="#prediction-reactive-contracts"/><reactive:paste select="#prediction-reactive-population"/></reactive:file>

## Fix observable anchors independently of either algorithm

These focused witnesses compare corrected kernels with separate oracles, show that
seeded faults fail held-out data, and assert concrete alias, group, ownership,
mailbox and index-shift outcomes. Per-case errors must not erase unrelated passes,
and JSON types must remain significant. These are harness checks, not model evidence.

<reactive:copy id="prediction-reactive-integrity">
"""Focused integrity checks for the frozen reactive semantic extractions."""
import json
import unittest
import bench
from reactive_cases import TASKS, dcl_oracle, flatten_oracle, groups_oracle, union_oracle, where_oracle

class ReactiveIntegrity(unittest.TestCase):
    def test_correct_kernels_match_independent_oracles(self):
        for task in TASKS:
            with self.subTest(task=task['id']):
                cases=task['public']+task['hidden']
                result=bench.score(task['solution'],cases)
                self.assertEqual(len(result['passed']),14)
                self.assertTrue(result['correct'],result.get('error',result['passed']))

    def test_faults_are_detectable_and_long_cases_are_held_out(self):
        for task in TASKS:
            with self.subTest(task=task['id']):
                self.assertFalse(bench.score(task['start'],task['hidden'])['correct'])
                public={json.dumps(case[0],sort_keys=True) for case in task['public']}
                hidden={json.dumps(case[0],sort_keys=True) for case in task['hidden']}
                self.assertFalse(public & hidden)
                self.assertEqual(len(hidden),12)
                self.assertTrue(all(len(case[0][0])>=40 for case in task['hidden']))

    def test_alias_callbacks_use_creation_order_with_current_offsets(self):
        args=[['source','a',[1]],['attach',0,'x','a'],['attach',0,'y','a'],['insert','a',1,[2]]]
        actual=flatten_oracle(args)
        self.assertEqual(actual['snapshots'][-1],[1,2,1,2])
        self.assertEqual(actual['changes'][-2:],[[3,'add',2,[2]],[3,'add',1,[2]]])

    def test_retained_groups_and_owner_multiplicity(self):
        result=groups_oracle([['add','a',2],['remove','a'],['add','b',5]],3)
        self.assertEqual(result['snapshots'][1],[[2,1,[]]])
        self.assertEqual(result['snapshots'][-1],[[2,1,[['b',5]]]])
        self.assertEqual(sum(e[1]=='group' for e in result['changes']),1)
        actions=[['source','a',[]],['attach','x','a'],['attach','y','a'],['insert','a',5],
                 ['detach','x'],['dispose'],['delete','a',5]]
        result=union_oracle(actions)
        self.assertEqual(result['changes'],[[3,'add',5]])
        self.assertEqual(result['snapshots'][-3:],[[5],[5],[5]])
        self.assertEqual(result['subscribed'],[])

    def test_queue_precedes_quiescence_and_shutdown_precedes_queue(self):
        actions=[['enqueue',['add',1]],['enqueue',['complete',1]],['enqueue',['add',2]],
                 ['enqueue',['data',2,9]],['poll'],['poll'],['shutdown'],
                 ['enqueue',['data',2,10]],['poll']]
        result=dcl_oracle(actions,True)
        self.assertEqual(result['polls'],[['emit',[9]],['pending'],['done']])
        self.assertEqual(result['remaining'],1)
        self.assertEqual(result['latest'],[[2,9]])

    def test_quiescence_fault_is_detected_without_any_shutdown_action(self):
        actions=[['enqueue',['add',1]],['poll'],['enqueue',['complete',1]],
                 ['enqueue',['add',3]],['enqueue',['data',3,10]],['poll']]
        task=next(t for t in TASKS if t['id']=='dcl-queued-quiescence')
        expected=dcl_oracle(actions,True)
        self.assertEqual(expected['polls'],[['pending'],['emit',[10]]])
        self.assertFalse(bench.score(task['start'],[[[actions,True],expected]])['correct'])

    def test_index_shift_removes_visible_items_right_to_left(self):
        result=where_oracle([['insert',0,[['a',0],['b',1]]],['insert',0,[['x',1]]]],2)
        self.assertEqual(result['snapshots'],[[['a',0],['b',2]],[]])
        self.assertEqual(result['changes'][-2:],[[1,'remove',1,['b',2]],[1,'remove',0,['a',0]]])

    def test_case_errors_do_not_erase_other_outcomes_and_types_are_checked(self):
        result=bench.score('def run(x):\n    return 1 // x if x == 0 else x\n',[[[0],0],[[2],2]])
        self.assertEqual(result['passed'],[False,True])
        self.assertFalse(bench.score('def run():\n    return True\n',[[[],1]])['correct'])

if __name__=='__main__':
    unittest.main()
</reactive:copy>

<reactive:file path="experiments/prediction-gate/test_reactive.py"><reactive:paste select="#prediction-reactive-integrity"/></reactive:file>

## Retain the separate protocol and scope

The task population changed at the user's request after the smoke ceiling. Preserve that history and freeze this version before paid calls.

<reactive:entry path="experiments/prediction-gate/reactive-plan.md">
# Reactive algorithm benchmark v4

For the maintainer running the next prediction-gate evaluation and writing up its limits.

The completed smoke pilot (2026-11-03 20:08:29) was valid: baseline and gate each
passed 3/3 tasks on their first proposals, with no prediction mismatches or blocked
saves. It established that the harness works and exposed a ceiling on those tasks.
The user then explicitly requested more complex LiveLinq and DynamicCombineLatest
data. This new task population is versioned separately; it must not be pooled with
the smoke pilot or described as the original preregistered main experiment.
The earlier invalid protocol-2 run also remains retained as an infrastructure failure.

## What the new suite exercises

These are self-contained Python semantic extractions, not native C#/Rx.NET or
Rust/Tokio integration tests. Starting implementations contain deliberately seeded
faults. They are benchmark repair exercises, not claims that the upstream projects
have those exact defects. The source references identify the inspected mechanisms;
the task text is the complete benchmark contract.

| Task | Interactions checked |
|---|---|
| DCL readiness and retained values | Pending sources, completed cached values, late data after removal/completion, numeric source order |
| DCL queued quiescence | FIFO draining, one emission per poll, queued additions behind completion, terminal state, shutdown before queued data |
| LiveLinq list flattening | Shared inner source aliases, callback creation order versus outer order, per-callback offsets, range removal, detachment and disposal |
| LiveLinq persistent grouping | Same-key value changes, duplicate source IDs, stable group identities through empty/repopulate cycles, stale updates |
| LiveLinq distinct ownership | Multiple owners of one inner source, overlapping values, zero/positive reference-count transitions, reattachment and frozen disposal |
| LiveLinq index-sensitive filtering | Range edits shifting every downstream index, predicate toggles, projection updates, canonical remove/add order and unsubscribed indices |

Each task has two public traces and twelve disjoint held-out traces: 12 public and
72 held-out cases across six task variants. Held-out traces have 44–79 actions,
from fixed seeds 1100–1111 (the second DCL variant offsets those seeds by 100).
Two DCL variants share an algorithm, so six variants are not six independent
algorithm families. Repeated trial sessions are also not new task types.

Public prediction questions require the exact complete output for both public
traces, including delta logs, state snapshots and final subscriptions where the
API returns them. The gate never receives held-out cases. The model receives only
the task's starting kernel, complete repair requirements, public inputs, restricted
Python interface and provenance references. Evaluator solutions, generators,
oracles, hidden inputs and scores stay outside its conversation and behind the
existing filesystem quarantine.

The LiveLinq reference is pinned to
[`a50ba1976b78b0edf635a96074d1895812141564`](https://github.com/nloum/LiveLinq/tree/a50ba1976b78b0edf635a96074d1895812141564).
Inspected boundaries include
[`SelectMany` offsets and disposal](https://github.com/nloum/LiveLinq/blob/a50ba1976b78b0edf635a96074d1895812141564/LiveLinq/Extensions.List.SelectMany.cs),
[`MovingRanges` invalidation](https://github.com/nloum/LiveLinq/blob/a50ba1976b78b0edf635a96074d1895812141564/LiveLinq/List/MovingRanges.cs),
[retained empty groups](https://github.com/nloum/LiveLinq/blob/a50ba1976b78b0edf635a96074d1895812141564/LiveLinq.Tests/Extensions.Set.GroupBy.Tests.cs),
[set flattening](https://github.com/nloum/LiveLinq/blob/a50ba1976b78b0edf635a96074d1895812141564/LiveLinq/Extensions.Set.SelectMany.cs), and
[index-sensitive predicates](https://github.com/nloum/LiveLinq/blob/a50ba1976b78b0edf635a96074d1895812141564/LiveLinq/Extensions.List.Where.cs).
The local reference is
[`DynamicCombineLatest`](../../crates/hick-flow/src/combine_latest.rs)
at repository revision `6662e6566f9df595747248382317291a92fb0400`.
ComposableCollections was an older home for LiveLinq and is not a second source.

## Keep reference answers independent

Correct kernels are compared against evaluator-only reference reducers. Collection
oracles recompute views from source state rather than calling the candidate's
incremental helpers; DCL uses a different record representation and mailbox cursor.
Hand-written witnesses additionally fix alias callback order, empty-group identity,
ownership multiplicity, queued additions, shutdown priority and index-shift deltas.
Agreement is evidence within this finite suite, not proof that either oracle is
universally correct. No model outcomes were used to choose the held-out seeds.

A runtime exception in one case now produces an explicit failed-case marker while
preserving other case outcomes. Timeouts or whole-module admission failures can
still fail an entire case batch; the receipt distinguishes them. JSON comparison
checks types as well as values, so `true` cannot silently satisfy an expected `1`.
The AST policy admits ordinary list/set mutation and lambdas, while retaining the
import/reflection/private-name restrictions. The exact builtin/method interface
is included in both conditions' task text. Observation output is bounded at 4 MiB.

## Freeze before live calls

The primary outcome and success rule are unchanged: all scheduled sessions count;
final accepted code must pass every held-out case; compare paired gate minus
baseline success rate; require at least +10 percentage points and a paired bootstrap
95% lower bound above zero for the declared success criterion. Anything else is
inconclusive for benefit, with failures and any deterioration reported explicitly.
A degenerate bootstrap interval from all ties does not demonstrate equivalence.

Both conditions use the same model and low reasoning effort, six response attempts,
360-second session budget, 60-second transport timeout, 96,000-byte cumulative
replayed-input budget, 32,000-byte accepted answer limit and candidate execution
limits. API mode additionally requests at most 8,192 output tokens per call; the
subscription CLI cannot enforce that generation-token cap. Its answer-size limit
is checked after generation and is not a credit cap. These larger limits apply
because the task population changed, before any reactive-suite model comparison.
Do not mix these runs with the old 180-second smoke profile.

Pilot: one repeat per task, six pairs / twelve sessions. Main: four repeats per
task, 24 pairs / 48 sessions. Validate transport, accounting, confinement and gate
execution in the pilot before starting main; do not select cases or alter success
criteria based on which condition wins. All six frozen variants remain in main,
including easy, failed or nondiscriminating ones. Any repair to the benchmark or
runner after new outcomes requires another version and rerunning both arms.

At standard Sol credit rates already documented in the runbook, a planning
scenario of 12,000 uncached input and 3,000 output tokens costs about 1.35 credits
per turn. That is about 16.2 credits for a one-turn-per-session pilot or 64.8 credits
for a one-turn main; six turns per session would be about 97.2 and 388.8 credits.
These are scenarios, not enforced spending caps or verified debits. CLI overhead,
reasoning, caching, retries and actual output sizes change usage. No model calls
are made by the offline commands below.

## Reproduce

Build the unchanged production output adapter if needed, then run only the focused
integrity checks:

```sh
cargo build --locked -p hick-literate --example prediction_bench
python3 -m unittest discover -s experiments/prediction-gate -p 'test_*.py' -v
```

Exercise the whole transport-independent harness without paid calls:

```sh
python3 experiments/prediction-gate/bench.py --simulated --suite reactive \
  --out target/prediction-gate-reactive-simulated
```

The model comparison runs from a normal Terminal using the existing ChatGPT sign-in:

```sh
python3 experiments/prediction-gate/bench.py --live --backend codex --suite reactive \
  --model gpt-6.1-sol --reasoning low --repeats 1 \
  --out "target/prediction-gate-reactive-pilot-$(date +%Y%m%d-%H%M%S)"
```

After validating that pilot, the main command changes `--repeats 1` to
`--repeats 4` and uses a fresh `reactive-main` output directory. Do not use the old
smoke command without `--suite reactive`: it intentionally retains the old tasks.
Preserve the manifest, actual parser warnings, every proposal/prediction/gate result,
held-out score and accepted final patch. The manifest hashes the selected fixture
module, runner, Codex transport and production gate binary and records all limits.
Keep output directories evaluator-only throughout all runs.

## What this can support in the blog

Report the old smoke pilot as an honest engineering checkpoint, then explain the
user-requested new population and larger shared budgets before presenting v4 data.
Show exact traces where a removed alias still affects an offset, an empty group
retains identity, or queued input changes a quiescence decision. Use actual archived
model proposals for claims about predictions and repairs; reference-kernel examples
are not model evidence. Report all tasks, first/final correctness, discordant pairs,
uncertainty, errors, retries and token/credit costs without pooling versions.

A pass demonstrates behavioral performance on deterministic extracted contracts.
It does not establish a mental model, native Rx.NET/Tokio correctness, scheduler
fairness, cancellation wake-up behavior, memory safety, thread races or realistic
large-project coding productivity. Snapshot-recomputing solutions can pass when
behavior is correct; asymptotic efficiency is not an outcome of this study.
</reactive:entry>


## Retain the completed Sol comparison without changing the benchmark

The main-run report records measured outcomes and limitations, including two
correct repairs refused for inconsistent structured predictions. This evidence
does not alter the frozen code, task population or success criterion.

<reactive:entry path="experiments/prediction-gate/sol-results.md">
# Sol reactive-v4 results: correct code blocked by incorrect predictions

For the author writing the prediction-gate blog post. This is the completed Sol
main comparison, not a pilot or simulated result. The declared success criterion
was not met. The gate showed no first-attempt coding improvement and refused two
implementations that passed every public and held-out check.

## Main outcome

The 24 paired observations comprise six fixed task variants with four repetitions
per variant: 48 isolated sessions using `gpt-6.1-sol`, low reasoning effort,
Codex CLI 0.166.0 and the frozen reactive-v4 limits. Both conditions used the same
starting code, repair contract, public inputs, tool restrictions and budgets;
the gate added prediction/sealing and acceptance requirements. The order was
randomized before execution. Pilots and the earlier invalid run are not pooled.

| Measurement | Baseline | Prediction gate |
|---|---:|---:|
| Sessions | 24 | 24 |
| First proposals passing all held-out cases | 24/24 | 24/24 |
| First-proposal held-out cases passed | 288/288 | 288/288 |
| Final accepted implementations passing all held-out cases | 24/24 | 22/24 |
| Final-state held-out cases passed | 288/288 | 266/288 |
| Completed tasks | 24 | 22 |
| Incorrect predictions | not applicable | 2 |
| Blocked saves of correct proposals | not applicable | 2 |
| Resubmission retries | 0 | 0 |
| Abandoned tasks | 0 | 2 |
| Regressions against initially passing cases | 0 | 0 |
| Completed CLI turns | 24 | 26 |
| Infrastructure/protocol failures | 0 | 0 |

The primary paired difference is **−8.33 percentage points** in final accepted
correctness (gate minus baseline). There were zero gate wins and two gate losses.
The predefined paired percentile bootstrap 95% interval is **[−20.83, 0.00]
percentage points**; the two-sided exact McNemar p-value is **0.5**. The required
+10-point improvement with a lower interval bound above zero was not achieved.

Those statistics describe this small fixed task set. Both losses occur in one
task variant, and the two DCL variants share code. Repetitions do not create new
algorithm families. With perfect observed baseline performance, the bootstrap is
also at a boundary: its zero upper endpoint does not prove that no benefit is
possible elsewhere. This is observed harm in these two sessions, not statistically
conclusive general harm or an equivalence finding.

## Where the two failures occurred

| Task variant | Baseline final | Gate final |
|---|---:|---:|
| DCL queued quiescence | 4/4 | 4/4 |
| DCL readiness and retention | 4/4 | 4/4 |
| LiveLinq distinct ownership | 4/4 | 2/4 |
| LiveLinq flattening aliases/offsets | 4/4 | 4/4 |
| LiveLinq index-sensitive filtering | 4/4 | 4/4 |
| LiveLinq persistent groups | 4/4 | 4/4 |

Both refused proposals correctly implemented the distinct-ownership kernel. Both
passed the public requirements and all twelve held-out traces. Their sealed JSON
predictions differed from the expected output in exactly one place: the final
subscription list in the second public trace.

```text
Predicted final subscriptions: ["y", "z"]
Required and produced result:  ["z"]
```

The trace detaches `y`, later attaches `z`, and finally detaches `x`. Both supplied
rationales correctly described that sequence and said that only `z` remained.
The structured predictions nevertheless included `y`. This is an observed
inconsistency between code, explanation and prediction; it does not identify the
model's internal cognitive cause.

The production ledger classified both attempts as `mismatch`, with
`original_requirement_satisfied=true` and `prediction_matched=false`. Promotion
was refused. Both sessions then returned `finish`; one explicitly explained that
its original prediction could not be changed after observation. Neither revised
or resubmitted the code. The accepted final file therefore remained the original
faulty starting implementation, rather than the correct proposed repair.

For this benchmark, the requirement and prediction each assert an exact output.
Once those two asserted values disagree, no code can satisfy both on that input
while preserving the original prediction. The observed abandonment is consistent
with that policy. It should not be reported as an inability to write correct code:
the model had already produced correct implementations by the benchmark's checks.

There were zero observed cases of incorrect code passing the gate, but there were
also **zero incorrect first proposals**. Therefore this run cannot estimate how
well the gate catches incorrect implementations. Its two false blocks are measured;
its incorrect-code detection sensitivity is unmeasured.

## Resource usage

| Measurement, totals per condition | Baseline | Prediction gate |
|---|---:|---:|
| Input tokens | 236,672 | 260,021 |
| Cached input tokens (included above) | 70,272 | 46,848 |
| Output tokens | 14,903 | 27,082 |
| Total input + output tokens | 251,575 | 287,103 |
| Sum of session elapsed times | 458.10 sec | 759.21 sec |
| Mean elapsed time per session | 19.09 sec | 31.63 sec |
| Estimated subscription credits | 12.22143 | 17.54627 |

The gate used **14.1% more total tokens**, **81.7% more output tokens**, and
**65.7% more elapsed time**. It consumed two additional CLI turns for the
post-refusal finish responses. These are descriptive differences for this run,
including prediction text, gate execution and the extra turns; they do not
separately identify the cost of each mechanism.

The credit estimate is **43.6% higher**, partly because the baseline received more
cache hits. At the same recorded rates with all input treated as uncached, the
estimates would be 15.55935 versus 19.77155 credits, a **27.1%** difference. This
normalization is supplementary context, not a revised primary outcome. Estimates
use the protocol's standard Sol rate assumptions; they are not verified account
debits. Actual internal provider request counts remain opaque: CLI turns and
measured token counters are reported instead of pretending those counts are known.

## Validity and evidence

The run directory is
`target/prediction-gate-reactive-main-20261103-214715`. It retains the manifest,
JSONL trial records, each initial/proposed/final implementation, every sealed
prediction and check result, private gate ledgers and all CLI event receipts.
The companion [machine-readable report](sol-results.json) records validation,
aggregate measurements, the predefined analysis and the raw JSONL digest.

The fixture module, runner and Codex transport hashes match the source snapshot
created before reactive live outcomes in
`target/prediction-gate-reactive-v4-source`. The production adapter binary also
matches. All 48 scheduled sessions and all 24 pairs are present. All submissions
were scored on twelve held-out cases; the six task variants therefore contribute
288 checks per condition, with repetitions reusing the same fixed cases.
No held-out/solution fields appeared in model-facing conversation records. CLI
receipts contain no native tool activity or unknown errors. All gate proposals
had a source-bound seal; passing proposals received promotion, and the two
mismatches did not. The 50 CLI turns completed without infrastructure failures.
These checks support this scoped protocol, not an adversarial audit of provider
internals or a universal host-isolation guarantee.

The original mechanism review, protocol, source links and reproduction commands
remain in [the runbook](README.md) and [the reactive plan](reactive-plan.md).
Do not edit the frozen tasks, limits or success criterion to make this result
look positive. Keep the raw held-out material evaluator-only during any further
model comparison; publishing results is a separate action.

## Blog-post outline with the actual result

1. **Hypothesis.** Committing observable behavior predictions before acceptance
   might improve code quality or catch mistakes. Separate those possibilities
   from claims about an AI possessing a mental model.
2. **What was built.** Explain the production seal/observe/promote ledger and
   withheld observations. Clarify that source edits themselves are permitted;
   this harness gates acceptance. The actual predicate API permits broad bounds,
   while this benchmark deliberately asks for exact complete public traces.
3. **Experimental design.** Describe the frozen matched baseline/gate conditions,
   isolated sessions, randomized order, identical budgets and held-out cases.
   Disclose the initial invalid parser run, the two valid pilot ceilings and the
   user-requested reactive task population. Keep pilots separate from main.
4. **Result.** All 48 first proposals passed held-out tests. Final accepted success
   was 24/24 versus 22/24. Present the −8.33-point difference, interval, p-value,
   raw counts and the fixed-task uncertainty limits together.
5. **Concrete failure.** Show the distinct-ownership trace, correct proposed patch,
   predicted `["y","z"]`, actual `["z"]`, correct prose explanation, mismatch
   receipt and subsequent finish. Explain why this exact-output gate could not
   recover through code changes while retaining the mistaken original prediction.
6. **Cost.** Show +14.1% total tokens and +65.7% elapsed time, plus raw input/cache/
   output counts and qualified credit estimates. Include the extra finish turns.
7. **What remains unknown.** No incorrect initial code was generated, so detection
   sensitivity was not exercised. These are small Python semantic extractions of
   LiveLinq/Hickory mechanisms, not native Rx.NET/Tokio or large-project trials.
   Repeated variants and shared DCL code limit generalization. Correctness on this
   suite does not prove understanding, and a wrong structured forecast does not
   by itself prove lack of understanding.
8. **Next study.** A Luna follow-up was suggested after the Sol pilot ceiling.
   Use the same frozen tasks and budgets, compare Luna baseline against Luna gate,
   and disclose that model choice as exploratory. Do not pool it with Sol or add
   harder cases until a separate version is registered. A future nonblocking-
   prediction arm could distinguish deliberation effects from rejection policy;
   it was not part of this experiment.

A defensible headline is: **“The AI fixed every task. The prediction gate still
blocked two correct fixes.”** A defensible conclusion is that this run found a
costly acceptance failure mode and no demonstrated correctness benefit, while
leaving broader effectiveness and cognitive explanations unresolved.
</reactive:entry>

<reactive:entry path="experiments/prediction-gate/sol-results.json">
{
  "kind": "sol-reactive-v4-main-analysis",
  "source_run": "target/prediction-gate-reactive-main-20261103-214715",
  "validation": {
    "bench.py_matches_frozen_snapshot": true,
    "bench.py_current_unchanged": true,
    "reactive_cases.py_matches_frozen_snapshot": true,
    "reactive_cases.py_current_unchanged": true,
    "codex_backend.py_matches_frozen_snapshot": true,
    "codex_backend.py_current_unchanged": true,
    "gate_binary_matches": true,
    "all_scheduled_sessions_recorded": true,
    "all_pairs_complete": true,
    "no_infrastructure_failures": true,
    "model_context_has_no_host_only_fields": true,
    "all_submit_proposals_scored_on_12_heldout_cases": true,
    "no_native_tool_activity_or_unknown_errors": true,
    "50_completed_cli_turns": true
  },
  "arms": {
    "baseline": {
      "input_tokens": 236672,
      "cached_input_tokens": 70272,
      "output_tokens": 14903,
      "elapsed_seconds": 458.09590108599986,
      "estimated_credits": 12.221429999999996,
      "blocked_saves": 0,
      "incorrect_predictions": 0,
      "retries": 0,
      "regressions": 0,
      "correct_code_blocked": 0,
      "cli_turn_attempts": 24,
      "sessions": 24,
      "first_correct": 24,
      "final_correct": 24,
      "completed": 24,
      "abandoned": 0,
      "total_tokens": 251575,
      "uncached_rate_normalized_credits": 15.55935
    },
    "gate": {
      "input_tokens": 260021,
      "cached_input_tokens": 46848,
      "output_tokens": 27082,
      "elapsed_seconds": 759.2081019130001,
      "estimated_credits": 17.54627,
      "blocked_saves": 2,
      "incorrect_predictions": 2,
      "retries": 0,
      "regressions": 0,
      "correct_code_blocked": 2,
      "cli_turn_attempts": 26,
      "sessions": 24,
      "first_correct": 24,
      "final_correct": 22,
      "completed": 22,
      "abandoned": 2,
      "total_tokens": 287103,
      "uncached_rate_normalized_credits": 19.77155
    }
  },
  "predefined_analysis": {
    "pairs": 24,
    "wins": 0,
    "losses": 2,
    "paired_risk_difference": -0.08333333333333333,
    "paired_bootstrap_95": [
      -0.20833333333333334,
      0.0
    ],
    "exact_mcnemar_p": 0.5,
    "simulated": false,
    "per_task": {
      "dcl-queued-quiescence": {
        "baseline": {
          "n": 4,
          "final_correct": 4,
          "failures": 0
        },
        "gate": {
          "n": 4,
          "final_correct": 4,
          "failures": 0
        }
      },
      "dcl-readiness-retention": {
        "baseline": {
          "n": 4,
          "final_correct": 4,
          "failures": 0
        },
        "gate": {
          "n": 4,
          "final_correct": 4,
          "failures": 0
        }
      },
      "livelinq-distinct-ownership": {
        "baseline": {
          "n": 4,
          "final_correct": 4,
          "failures": 0
        },
        "gate": {
          "n": 4,
          "final_correct": 2,
          "failures": 0
        }
      },
      "livelinq-flatten-alias-offsets": {
        "baseline": {
          "n": 4,
          "final_correct": 4,
          "failures": 0
        },
        "gate": {
          "n": 4,
          "final_correct": 4,
          "failures": 0
        }
      },
      "livelinq-index-sensitive-filter": {
        "baseline": {
          "n": 4,
          "final_correct": 4,
          "failures": 0
        },
        "gate": {
          "n": 4,
          "final_correct": 4,
          "failures": 0
        }
      },
      "livelinq-persistent-groups": {
        "baseline": {
          "n": 4,
          "final_correct": 4,
          "failures": 0
        },
        "gate": {
          "n": 4,
          "final_correct": 4,
          "failures": 0
        }
      }
    },
    "admitted_proposals": 48,
    "primary_outcome_available": true,
    "conclusion": "inconclusive; not evidence of no benefit"
  },
  "raw_records_sha256": "d4c45a9e78e04516dcccdd2f822d04800c55bc977b3e19ef8338e1cff1a26529",
  "limitations": [
    "Six fixed task variants; two share DCL code",
    "Python semantic extractions, not native runtime tests",
    "All initial proposals correct: no erroneous-code detection sensitivity estimate",
    "Both false blocks occur in one task variant",
    "CLI internal provider call counts opaque",
    "Credit estimates are not actual account debits and caching differs",
    "Inference across new task populations is not established"
  ],
  "primary_outcome": "final accepted implementation passes every held-out case",
  "success_criterion_met": false,
  "rate_assumptions_credits_per_million": {
    "input": 50,
    "cached_input": 2.5,
    "output": 250
  },
  "frozen_source_snapshot": "target/prediction-gate-reactive-v4-source/snapshot.json"
}
</reactive:entry>

</reactive:doc>
