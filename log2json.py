import json;
import re;
import sys;

recap = []

def parse_sm():
    state = "idle"
    ret = []
    op = None
    line = yield
    while True:
        if line == None:
            break
        if state == "idle":
            if "insert key" in line:
                key_val = line.replace("insert key: ", "").split()
                op = { 'insert': key_val }
                state = "on"
            line = yield
        if state == "on":
            if "dump" in line:
                kvs = line.replace("dump: ", "").split(";")
                kvs.pop()
                op["dump"] = list(map(lambda kv: kv.split(": "), kvs))
            if "send chunk" in line:
                # todo: work on the regex to accept deletes
                updates = re.findall("Update\((\d+), (\d+)\)", line)
                op["chunk"] = updates
            if "forward update" in line:
                forwards = re.findall("insert:\s(\d*)\s(\d*)", line)
                op["forwards"] = forwards
            if "insert key" in line:
                state = "idle"
                ret.append(op)
                op = None
                continue
            line = yield
    if not op is None:
        ret.append(op)
    yield ret

with open(sys.argv[1]) as logs:
    p = parse_sm()
    next(p)
    for line in logs.readlines():
        p.send(line)
    print(f"const {sys.argv[2]} = '{json.dumps(p.send(None))}'")
