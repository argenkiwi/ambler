The goal of this project is to provide a starting point to build state machines in different programming languages with the help of an LLM by using a simple and predictable design pattern.

Every Ambler application begins with a call to the `amble` function, which receives 3 parameters:
- an initial state;
- a initial node identifier;
- a function which receives a node identifier, calls the corresponding node function and returns the results.

Node identifiers should be represented as a finite group, like enums or sealed interfaces depending on the programming language. Node functions receive the current state as a parameter and return the updated state and an optional node identifier for the next node function to be called. The amble function will finish when no node identifier is returned.

At the root of the project there should be SPECS.md file containing the specifications of a sample application to be implemented in each programming language. The specifications should consist of an description of what sample application does followed by an explanation of what each node does and to which node it will transition to depending on the outcome of the perfomed operation.

There should be a folder for each programming language of interest and they should all contain:
- an implementation of the amble function;
- an implementation of the sample application; 
- a README.md file containing instructions on how to run the sample application.
