# Insert randoms values in database on 127.0.0.1:3000

import requests;
import random;
import time

for _ in range(0, 100):
    key = random.randint(0,100)
    value = random.randint(0,999)
    requests.post('http://127.0.0.1:3000/insert', json=[{'Update': [f'{key}', f'{value}']}])
    #time.sleep(0.10)
