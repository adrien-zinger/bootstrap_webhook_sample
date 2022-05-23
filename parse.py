dic = {}
with open("serv_0.log") as file:
    for line in file.readlines():
        if "dump: " in line:
            dic = {}
            line = line.replace("dump: ", "")
            for kv in line.split(';'):
                kv = kv.split(':')
                if len(kv) == 2: dic[kv[0]] = kv[1]

print(dic)

print("\n")
lserv0 = {}
with open("serv_0.log") as file:
    for line in file.readlines():
        if "dump: " in line:
            lserv0 = {}
            line = line.replace("dump: ", "")
            for kv in line.split(';'):
                kv = kv.split(':')
                if len(kv) == 2: lserv0[kv[0]] = kv[1]

print(lserv0)
