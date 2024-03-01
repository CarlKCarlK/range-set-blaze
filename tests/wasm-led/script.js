import init, { get_led_state_and_duration } from './pkg/wasm_led.js';

let customTime = 0; // Initialize your custom time to 0
let movieIndex = 0; // Initialize the movie index
let timeoutHandle = null; // Store the timeout handle so it can be cleared

function endCurrentTimeout() {
    if (timeoutHandle !== null) {
        clearTimeout(timeoutHandle);
        timeoutHandle = null;
    }
}

function updateLedState() {
    console.log("updateLedState called with customTime:", customTime); // Check if and when this function is called

    // Call the WASM function with the current custom time
    console.log("movieIndex:", movieIndex); // Debug print the movie index
    console.log("customTime:", customTime); // Debug print the custom time
    const ledStateResult = get_led_state_and_duration(movieIndex, customTime);
    const ledState = ledStateResult.state; // Access the state field
    const duration = ledStateResult.duration; // Access the duration field

    // Debug print the ledState
    console.log("LED State:", ledState);

    // Update the LED segments based on the ledState
    for (let i = 0; i < 7; i++) {
        const segmentId = `seg-${String.fromCharCode(97 + i)}`; // seg-a, seg-b, ..., seg-g
        const isOn = (ledState & (1 << i)) !== 0;
        document.getElementById(segmentId).style.backgroundColor = isOn ? 'red' : 'lightgrey';
    }

    // Update the decimal point separately if needed
    const isDecimalOn = (ledState & 0b10000000) !== 0; // Check the 8th bit for the decimal point
    document.getElementById("dec").style.backgroundColor = isDecimalOn ? 'red' : 'lightgrey';

    // Increment your custom time by the duration returned from WASM
    customTime += duration;

    // Set a timeout to call this function again after the specified duration
    timeoutHandle = setTimeout(updateLedState, duration);
}

function onMovieItemClick(event) {
    // Determine which movie was clicked based on the text content of the clicked element
    const movieName = event.target.textContent;
    switch (movieName) {
        case 'Hello World':
            movieIndex = 0;
            break;
        case 'Circles':
            movieIndex = 1;
            break;
        case 'Count Down':
            movieIndex = 2;
            break;
        // Add more cases as needed for additional movies
        default:
            movieIndex = 0; // Default to the first movie
            break;
    }

    // Reset the custom time
    customTime = 0;

    // End any running setTimeout
    endCurrentTimeout();

    // Call updateLedState to reflect the new movie
    updateLedState();
}

document.addEventListener('DOMContentLoaded', (event) => {
    init().then(() => {
        console.log("WASM initialization complete. Starting LED state update.");
        updateLedState();

        // Add click event listeners to all movie items
        const menuItems = document.querySelectorAll('#menu div');
        menuItems.forEach((item, index) => {
            item.addEventListener('click', onMovieItemClick);
            // Optionally, you could use the index directly: movieIndex = index;
        });
    }).catch((error) => {
        console.error("WASM initialization failed:", error);
    });
});
