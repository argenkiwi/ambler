## Program Specifications

This program is a simple counter application that demonstrates the use of `ambler.py` for defining program stages and transitions.

### Stages (Nodes)

- `PROMPT_NUMBER`: Prompts the user to enter a starting number for the count. This is the initial stage of the application.
- `START`: Initializes the counting process with the number provided by the user.
- `STEP`: Increments the counter by one and prints the current count. It randomly decides whether to continue counting or stop.
- `STOP`: Terminates the counting process.

### Shared State

The shared state is an integer representing the current count. It is initialized to `0` but is updated by the `PROMPT_NUMBER` stage based on user input.

### Flow

1. The application starts at the `PROMPT_NUMBER` stage.
2. In `PROMPT_NUMBER`, the user is prompted to enter an integer. If valid, this number becomes the initial state, and the flow transitions to `START`.
3. In `START`, a message indicating the start of counting from the user-provided number is printed, and the flow transitions to `STEP`.
4. In `STEP`, the current count is incremented and printed. There's a random chance to either continue to `STEP` again or transition to `STOP`.
5. In `STOP`, a final message is printed, and the application terminates.
