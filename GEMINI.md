# Guidelines

## Goal
The goal of this project is to provide a starting point for building state machines in diverse programming languages with the help of an LLM by using a simple and predictable design pattern.

## Design Pattern
- Every Ambler application begins with a call to the `amble` function, which receives a function as a parameter; the content of the function should consist of a call to the first step or node function passing the intial state as a parameter.
- Every step or node function will receive the current state and return an instance of Next which receives a function as a parameter; the content of the function should consist of a call to the next step or node function passing the updated state as a parameter.

## Structure
At the root of the project there should be a SPECS.md file containing the specifications of a sample application to be implemented in each programming language. The specifications should consist of an description of what the sample application does followed by an explanation of the role of each node and, depending on the outcome of the perfomed operation, to which other node they transition to.

There should be a folder for each programming language of interest and they should all contain:
- an implementation of the amble function;
- an implementation of the sample application; 
- a README.md file containing instructions on how to run the sample application.
- a GEMINI.md file with guidelines on how to structure an Ambler application specific to the given programming language.
