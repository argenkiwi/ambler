# Guidelines

This document describes conventions to be followed to build _Ambler_ applications.

## Components

- State: represents any data to be shared across the application.
- Step: a function that receives the current state as a parameter, performs a specific task and returns an optional `Next`.
- Next: a class with a reference to a step function and the current state.
- Amble: a function that takes the initial step function and state as parameters and continues to execute each step until no `Next` is returned.

## Process

1. Identify the application's shared state.
2. Identify the initial step and state.

## Structure
At the root of the project there should be:
- A SPECS.md file containing the specifications of an application to be implemented. The specifications should consist of an description of what the application does followed by an explanation of each step and, depending on the outcome of the performed operation, to which other step they transition to next.
- A README.md file containing instructions on how to run the application.
