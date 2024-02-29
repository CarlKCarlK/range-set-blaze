document.addEventListener('DOMContentLoaded', () => {
    const segments = document.querySelectorAll('.segment, .decimal');

    function changeSegmentColor() {
        const randomSegmentIndex = Math.floor(Math.random() * segments.length);
        const segment = segments[randomSegmentIndex];
        segment.style.backgroundColor = segment.style.backgroundColor === 'red' ? 'black' : 'red';
    }

    setInterval(changeSegmentColor, 1000);
});
