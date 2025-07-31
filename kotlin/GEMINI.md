# Guidelines

## Goal
The goal of this project is to provide a starting point for building state machines in diverse programming languages with the help of an LLM by using a simple and predictable design pattern.

## Design Pattern
- An Ambler application consists of a series of steps (or nodes) expressed as functions that receive a state as a parameter and return an optional instance of the `Next` class.
- Tne `Next` class represents the next step function to be called and the current state.
- Every Ambler application begins with a call to the `amble` function, which receives the initial step function and state as parameters.

## Structure
At the root of the project there should be:
- A SPECS.md file containing the specifications of an application to be implemented. The specifications should consist of an description of what the application does followed by an explanation of each step and, depending on the outcome of the performed operation, to which other step they transition to next.
- A README.md file containing instructions on how to run the application.
