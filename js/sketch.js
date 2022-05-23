const log_sm = {
    pos: 0,
    max: 0,
    play: true,
    serv_0: { pos: {x: 1, y: 20}, strokeColor: "#7a9de0" },
    serv_1: { pos: {x: 1, y: 100}, strokeColor: "#27dd95" },
    serv_2: { pos: {x: 1, y: 180}, strokeColor: "#fc6d47" },
    client: { pos: {x: 1, y: 260}, key_vals: {}, cache: {} },

    init() {
	this.serv_0.logs = JSON.parse(log_serv_0);
	Object.setPrototypeOf(this.serv_0, serv);
	this.serv_1.logs = JSON.parse(log_serv_1);
	Object.setPrototypeOf(this.serv_1, serv);
	this.serv_2.logs = JSON.parse(log_serv_2);
	Object.setPrototypeOf(this.serv_2, serv);
	this.max = this.serv_0.logs.length;
	Object.setPrototypeOf(this.client, client);
    },

    incr() {
	if (this.play && this.pos < this.max - 1)
	    this.pos++;
    },

    decr() {
	if (this.pos > 0)
	    this.pos--;
    },

    /// Show all server and client
    show() {
	fill(0);
	//noStroke();
	//text(`Insert/Update ${this.serv_0.logs[this.pos].insert}`, 50, 30);
	this.serv_0.show(this.pos);
	this.client.concat(this.serv_0.logs[this.pos].chunk, this.serv_0.strokeColor);
	this.client.forward(this.serv_0.logs[this.pos].forwards, this.serv_0.strokeColor);
	this.serv_1.show(this.pos);
	this.client.concat(this.serv_1.logs[this.pos].chunk, this.serv_1.strokeColor);
	this.client.forward(this.serv_1.logs[this.pos].forwards, this.serv_1.strokeColor);
	this.serv_2.show(this.pos);
	this.client.concat(this.serv_2.logs[this.pos].chunk, this.serv_2.strokeColor);
	this.client.forward(this.serv_2.logs[this.pos].forwards, this.serv_2.strokeColor);
	this.client.show(this.pos);
    },
};

const client = {
    show(pos) {
	if (this.cache[pos] === undefined) {
	    this.cache[pos] = JSON.stringify(this.key_vals);
	    this.show_kv(this.key_vals);
	} else {
	    this.show_kv(JSON.parse(this.cache[pos]));
	}
    },

    show_kv(key_vals) {
	let keys = Object.keys(key_vals).sort();
	for (let i = 0; i < keys.length; ++i) {
	    let o = key_vals[keys[i]];
	    let color = "#" + map(o.val, 0, 999, 0, 999999);
	    stroke(o.strokeColor);
	    fill(color);
	    let x = (i * 20) % 600;
	    rect(this.pos.x + x, this.pos.y + Math.floor(i * 20 / 600) * 20, 20, 18);
	}
    },

    concat(arr, strokeColor) {
	if (arr === undefined) return;
	arr.forEach(a => {
	    this.key_vals[a[0]] = { val: Number.parseInt(a[1]), strokeColor: strokeColor };
	});
    },

    forward(kv, strokeColor) {
	if (kv === undefined) return;
	this.key_vals[kv[0][0]] = { val: Number.parseInt(kv[0][1]), strokeColor: strokeColor };
    }

};

const serv = {
    /// show serv at the state machine `pos` as position
    show(pos) {
	let log = this.logs[pos];
	// show stack
	if (log === undefined)
	    console.warn(`log undefined at pos ${pos}`)
	for (let i = 0; i < log.dump.length; ++i) {
	    let color = "#" + map(log.dump[i][1], 0, 999, 0, 999999);
	    if (this.logs[pos].insert[0] === log.dump[i][0])
		stroke("red");
	    else stroke(this.strokeColor);

	    fill(color);
	    let x = (i * 20) % 600;
	    rect(this.pos.x + x, this.pos.y + Math.floor(i * 20 / 600) * 20, 20, 18);
	}
    },
};

function setup() {
    createCanvas(600, 800);
    frameRate(3);
    strokeWeight(2);
    log_sm.init();
}

function draw() {
    clear();
    log_sm.show();
    log_sm.incr();
}
