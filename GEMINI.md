# Guidelines

This document describes conventions to be followed to build _Ambler_ applications.

## Process

1. Identify the application's shared state.
2. Identify the application's steps.
3. Define the leads to each step and implement the steps' logic.
4. Call the ambler function at the start of the application providing the initial state and lead as well as a `follow` function.

## Structuree 
At the root of the project there should be:
- A SPECS.md file containing the specifications of an application to be implemented. The specifications should consist of an description of what the application does followed by an explanation of each step and, depending on the outcome of the performed operation, to which other step they transition to next.
- A README.md file containing instructions on how to run the application.
