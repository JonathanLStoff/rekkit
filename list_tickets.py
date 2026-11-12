import json, glob
tickets = []
for f in sorted(glob.glob("planning/mods/redpower_2.tickets/part*.json")):
    with open(f) as j_file:
        tickets.extend(json.load(j_file)["tickets"])
print(f"Total tickets: {len(tickets)}")
for t in tickets:
    print(f"{t['key']} - {t['title']}")
