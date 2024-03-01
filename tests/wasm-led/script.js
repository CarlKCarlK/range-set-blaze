import init, { get_led_state_and_duration } from './pkg/wasm_led.js';

let customTime = 0; // Initialize your custom time to 0

async function updateLedState() {

    console.log("updateLedState called with customTime:", customTime); // Check if and when this function is called

    // Call the WASM function with the current custom time
    const ledStateResult = get_led_state_and_duration(customTime);
    const ledState = ledStateResult.state; // Access the state field
    const duration = ledStateResult.duration; // Access the duration field

    // Debug print the ledState cmk
    console.log("LED State:", ledState);


    // Update the LED segments based on the ledState
    for (let i = 0; i < 7; i++) {
        const segmentId = `seg-${String.fromCharCode(97 + i)}`; // seg-a, seg-b, ..., seg-g
        // console.log("segmentId:", segmentId);
        const isOn = (ledState & (1 << i)) !== 0;
        document.getElementById(segmentId).style.backgroundColor = isOn ? 'red' : 'black';
    }

    // Update the decimal point separately if needed
    const isDecimalOn = (ledState & 0b10000000) !== 0; // Check the 8th bit for the decimal point
    document.getElementById("dec").style.backgroundColor = isDecimalOn ? 'red' : 'black';

    // Increment your custom time by the duration returned from WASM
    customTime += duration;

    // Set a timeout to call this function again after the specified duration
    setTimeout(updateLedState, duration);
}

init().then(() => {
    console.log("WASM initialization complete. Starting LED state update.");
    updateLedState();
}).catch((error) => {
    console.error("WASM initialization failed:", error);
});