
let angle = 0;
let interval;

self.onmessage = function (event) {
    console.log("worker event.data", event.data)
    if (event.data.start) {
        interval = setInterval(rotate, 100)
    }
    else {
        clearInterval(interval)
    }
}

function rotate() {
    // console.log("worker rotate")
    angle += 4;
    angle %= 360;
    self.postMessage(angle);
}
