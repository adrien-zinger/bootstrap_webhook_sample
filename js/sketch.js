const log_sm = {
    pos: 0,
    serv_0: {},
    //serv_1: {},
    //serv_2: {},

    init() {
	this.serv_0.logs = JSON.parse(log_serv_0);
	Object.setPrototypeOf(this.serv_0, serv);
	this.serv_0.pos = {x: 50, y: 50};
    },

    show() {
	this.serv_0.show(this.pos);
    }
};

const serv = {
    /// show serv at the state machine `pos` as position
    show(pos) {
	let log = this.logs[pos];
	// show stack
	console.log(log);
	for (let i = 0; i < log.dump.length; ++i) {
	    let color = "#" + map(log.dump[i][1], 0, 999, 0, 999999);
	    fill(color);
	    rect(this.pos.x + i * 20, this.pos.y, 20, 20);
	    //rect(20, 20, 200, 200);
	}
    }
};

function setup() {
    createCanvas(600, 400);
    frameRate(1);
    log_sm.init();
}

function draw() {
    clear();
    log_sm.show();
    log_sm.pos++;
}
