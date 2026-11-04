"""Frozen reactive-v4 fixtures. Evaluator-only; never expose solutions or held-out cases."""
import random

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

def groups_trace(seed, length=55):
    rng=random.Random(seed)
    trace=[['add','a',2],['add','b',5],['value','a',8],['remove','b'],['add','c',5]]
    for _ in range(length):
        op=rng.choice(['add','add','remove','value']);token=rng.choice(['a','b','c','d','e'])
        trace.append([op,token] if op=='remove' else [op,token,rng.randrange(-10,11)])
    if seed%2:trace += [['dispose',None],['add','late',9],['value','a',42]]
    return [trace,3 if seed%2 else 4]

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
