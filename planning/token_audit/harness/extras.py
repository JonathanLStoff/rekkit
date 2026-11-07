import json, glob, collections, re, os
per = {}
for f in sorted(glob.glob('runs/*.stream.jsonl')):
    rid = os.path.basename(f).replace('.stream.jsonl', '')
    if rid == 'pilot': continue
    seen = collections.Counter()
    for l in open(f, encoding='utf-8', errors='replace'):
        try: s = json.loads(l).get('step_update') or {}
        except ValueError: continue
        if s.get('state') != 'DONE': continue
        if s.get('tool_name') == 'read_file':
            p = s['tool_info']['parameters']; path = str(p.get('Path') or list(p.values())[0])
            seen['READ ' + re.sub(r'^.*rekkit-audit-[^\/]+[\/]', '', path)] += 1
        if s.get('tool_name') == 'run_command':
            seen['CMD ' + s['tool_info']['parameters'].get('CommandLine', '')[:70]] += 1
    d = {k: v for k, v in seen.items() if v > 1}
    if d: per[rid] = d
for r, d in per.items(): print(r, d)
tb = tn = 0
for f in sorted(glob.glob('runs/*.json')):
    d = json.load(open(f))
    if 'per_turn' not in d or d['id'] == 'pilot': continue
    ctx = [r[0] + r[1] for r in d['per_turn']]
    tb += sum(ctx); tn += max(ctx[-1] - ctx[0], 0)
    print(d['step'], 'ctx', ctx[0], '->', ctx[-1], 'billed/net-growth x', round(sum(ctx) / max(ctx[-1] - ctx[0], 1), 1))
print('total billed', tb, 'total net new context', tn, 'ratio', round(tb / tn, 1))
