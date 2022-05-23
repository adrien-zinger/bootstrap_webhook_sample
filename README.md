# Webhook bootstrap implementation

Sample webhook strategy for real time database duplication.

Linked paper: [github markdown](https://github.com/adrien-zinger/presentations/blob/main/streaming_bootstrap/streaming.md)

The goal of the repository is to present how to implement a real time
bootstrap in a network that guaranty a data consistency.

The implementation is explained on the paper bellow.

## Usage

You can test manually the implementation by running some python scripts in the
repository.

First start a server with the command line. The server accept multiple http
requests described in the linked paper.

```bash
cargo run -- 3000
```

Open a new terminal and keep the previous open. Then in a python script,
like in `insert.py`, define your insertion strategy. You can define a script
that:
- keep up to date one or more servers
- insert / delete randomly
- infinite range, insfinite values

... You're free to imagin the scenario you wish.

Basic example:
```py
# Perhaps 5 seconds of random insertions
import requests;
import random;
import time

for _ in range(0, 100):
    key = random.randint(0,100)
    value = random.randint(0,999)
    requests.post('http://127.0.0.1:3000/insert', json=[{'Update': [f'{key}', f'{value}']}])
    time.sleep(0.02)
```

Then you can start the bootstraping node. The second port is the port of the original
server to duplicate.

```bash
cargo run -- 3001 3000
```

If you want to run both, insertion script and bootstrapper server you can launch the
commands in parallel.

```bash
# start one server
python insert_slow.py & cargo run -- 3001 3000

# multiple servers
python insert_slow.py &
cargo run -- 3001 3000 &
cargo run -- 3002 3000 &
cargo run -- 3003 3002 # Ok, bootstrap from a bootstraper!
```

Finally, you can check the consistency closing the servers with `ctrl-c`, the servers
will dump in the output the data container `key - value`

## Data visualization

You can reproduce the data visualization for a better comprehension of the
algorithm.

First, run the script:

```sh
python ./insert.py
```

While the script is running, run the server executable like that:

```sh
target/release/server 3003 3000 3001 3002 > client.log
```

The bootstrap will start for the node `127.0.0.1:3003` with the synchronized
nodes on ports `3000`, `3001` and `3002`. To be realists, the nodes are a bit
desynchronized.

Then before enjoying a strange and obscure data visualization if you don't know
the subject, please generate the new `js` files with

```sh
python log2json.py serv_0.log log_serv_0 > logs/serv_0.js
python log2json.py serv_1.log log_serv_1 > logs/serv_1.js
python log2json.py serv_2.log log_serv_2 > logs/serv_2.js
```

Then you can open the visualization.html file in your favourite browser and
look at the magic.

## Next steps

You can contribute to the repository with any PR you want, create issues to ask
a question about the algorithm and his implementation.

1. You can participate to the current repository by reading the linked paper
   and implementing the missing things. Ask questions in the github issue
   section to increment the FAQ.
2. Add a CI for an automatically test report. Definition of scenarios with
   a P2P situation (bootstrap from multiple nodes) and casual scenarios.
   Identify corner cases.
