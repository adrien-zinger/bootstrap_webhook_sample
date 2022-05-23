# Insert randoms values in database on 127.0.0.1:3000

import requests
import random
import time
import subprocess
import os
import signal

serv_ports = [3000, 3001, 3002]
serv_procs = []

def fill_servers(n, dump_cli):
    for _ in range(0, n):
        # 100 possible different keys (favorize the colisions for the example)
        key = random.randint(0,100)
        value = random.randint(0,999)
        print(f"client send {key}:{value}")
        for serv_port in serv_ports:
            requests.post(f'http://127.0.0.1:{serv_port}/insert',
                    json=[{'Update': [f'{key}', f'{value}']}])
            print(f"sent to {serv_port}")
            requests.post(f'http://127.0.0.1:{serv_port}/dump')
            print(f"dump {serv_port}")
            time.sleep(0.001)
            if dump_cli:
                requests.post('http://127.0.0.1:3300/dump')
        time.sleep(0.05)

# start servers
for (i, serv_port) in enumerate(serv_ports):
    serv_procs.append(
        subprocess.Popen(
            f"target/release/server {serv_port} > serv_{i}.log",
            stdout=subprocess.PIPE,
            shell=True,
            preexec_fn=os.setsid
        )
    )

# init a little bit the bootstrap servers
fill_servers(10, False)

print("Start client!")
# start the client!!
# client = subprocess.Popen(
#     f"target/release/server 3300 3000 3001 3002 > client.log",
#     stdout=subprocess.PIPE,
#     shell=True,
#     preexec_fn=os.setsid
# )

# time.sleep(1)

# Hop we continue to fill the servers :)
fill_servers(100, False)

# for serv in serv_procs:
#    os.killpg(os.getpgid(serv.pid), signal.SIGTERM)

# os.killpg(os.getpgid(client.pid), signal.SIGTERM)
