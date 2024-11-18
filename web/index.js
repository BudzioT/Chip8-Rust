import init, * as wasm from "./wasm.js"

const WIDTH = 64;
const HEIGHT = 32;
const SCREEN_SCALE = 15;

const TICKS_PER_FRAME = 10;
let current_frame = 0;

// Canvas as display for emulation
const canvas = document.getElementById("canvas");
canvas.width = WIDTH * SCREEN_SCALE;
canvas.height = HEIGHT * SCREEN_SCALE;

// Refresh the context of canvas
const ctx = canvas.getContext("2d")
ctx.fillStyle = "black";
ctx.fillRect(0, 0, canvas.width, canvas.height);

const input = document.getElementById("fileInput");
const fileList = document.getElementById("fileList");
const description = document.getElementById("description");

run().catch(console.error);

// Emulation loop
function programLoop(emu) {
    // Run the emulation
    for (let i = 0; i < TICKS_PER_FRAME; i++) {
        emu.tick();
    }
    emu.time_tick();

    // Clean the screen
    ctx.fillStyle = "black";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Draw the rest of screen
    ctx.fillStyle = "white";
    emu.draw_display(SCREEN_SCALE);
    current_frame = window.requestAnimationFrame(() => {
        programLoop(emu);
    });
}

async function run() {
    await init();
    let emu = new wasm.EmulatorWasm();

    document.addEventListener("keydown", function(event) {
        emu.keypress(event, true);
    })
    document.addEventListener("keyup", function(event) {
        emu.keypress(event, false);
    })

    // Load new game into emulator
    input.addEventListener("change", function(event) {
        // If there was already a file running, stop the animation
        if (current_frame !== 0) {
            window.cancelAnimationFrame(current_frame);
        }

        // Try to get the file
        let file = event.target.files[0];
        if (!file) {
            alert("Failed to get the file");
            return;
        }

        // Read the file, load it into the emulator
        let fileReader = new FileReader();
        fileReader.onload = function(event) {
            let buffer = fileReader.result;
            const rom = new Uint8Array(buffer);
            emu.reset();
            emu.load_data(rom);
            // Begin emulation
            programLoop(emu);
        }

        fileReader.readAsArrayBuffer(file);
    }, false)

    // Load file from the list
    fileList.onchange = (event) => {
        const fileName = event.target.value;
        const filePath = `./games/${fileName}`;

        changeDescription(fileName);

        // If there was already a file running, stop the animation
        if (current_frame !== 0) {
            window.cancelAnimationFrame(current_frame);
        }
        emu.reset();

        fetch(filePath)
            .then(response => response.arrayBuffer())
            .then(arrayBuffer => {
                const uint8Array = new Uint8Array(arrayBuffer);
                emu.load_data(uint8Array);
                programLoop(emu);
            }).catch(error => console.error("Error loading file:", error));
    };
}

function changeDescription(game) {
    let desc = "<p><h3>" + game + "</h3><b>Keybinds:</b>";
    if (game === "TETRIS") {
        desc +=
            "<ul>" +
            "<li><i>W / E</i> - move block</li>" +
            "<li><i>Q</i> - rotate block</li>" +
            "<li><i>A</i> - make it fall faster</li>" +
            "</ul></p>";
    }
    else if (game === "INVADERS") {
        desc +=
            "<ul>" +
            "<li><i>Q / E</i> - movement</li>" +
            "<li><i>W</i> - shoot</li>" +
            "</ul></p>";
    }
    else if (game === "PONG" || game === "PONG2") {
        desc +=
            "<ul>" +
            "<li><i>1 / Q</i> - Player 1 up/down movement</li>" +
            "<li><i>4 / R</i> - Player 2 up/down movement</li>" +
            "</ul></p>";
    }
    else if (game === "WIPEOFF") {
        desc +=
            "<ul>" +
            "<li><i>Q/E</i> - movement</li>" +
            "</ul></p>";
    }
    else if (game === "TANK") {
        desc +=
            "<ul>" +
            "<li><i>2 / S</i> - vertical movement</li>" +
            "<li><i>Q / E</i> - horizontal movement</li>" +
            "<li><i>W</i> - shoot</li>" +
            "</ul></p>";
    }
    else if (game === "TICTAC") {
        desc = "<p><h3>TIC TAC TOE</h3><b>Keybinds:</b>" +
            "<ul>" +
            "<li>1 / 2 / 3 - top row</li>" +
            "<li><i>Q / W / E</i> - middle row</li>" +
            "<li><i>A / S / D</i> - bottom row</li>" +
            "</ul>";
    }
    else if (game === "UFO") {
        desc +=
            "<ul>" +
            "<li><i>W</i> - shoot forward</li>" +
            "<li><i>Q</i> - shoot to the left</li>" +
            "<li><i>E</i> - shoot to the right</li>" +
            "</ul></p>";
    }
    else if (game === "BLINKY") {
        desc = desc = "<p><h3>BLINKY</h3><b>I don't know what's going on, but</br></b>" +
            "it's something like PacMan";
    }
    else if (game === "VERS") {
        desc = "<p><h3>VERS</h3><b>I don't know what's going on, but</br></b>" +
            "Something like Snake, but two???";
    }

    description.innerHTML = desc;
}
