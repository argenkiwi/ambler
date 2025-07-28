import random
import time
from enum import Enum, auto

from ambler import amble


# Nodes.
class Node(Enum):
    PROMPT_NUMBER = auto()
    START = auto()
    STEP = auto()
    STOP = auto()


def prompt_number(state):
    try:
        num = int(input("Enter a number to start counting from: "))
        return num, Node.START
    except ValueError:
        print("Invalid input. Please enter an integer.")
        return state, Node.PROMPT_NUMBER


def start(state):
    print(f"Let's count from {state}...")
    return state, Node.STEP


def step(state):
    count = state + 1
    time.sleep(1)
    print(f"...{count}...")
    return count, Node.STEP if random.choice([True, False]) else Node.STOP


def stop(state):
    print("...stop.")
    return state, None


# Flow.
node_functions = {
    Node.PROMPT_NUMBER: prompt_number,
    Node.START: start,
    Node.STEP: step,
    Node.STOP: stop,
}


def direct(state, node):
    return node_functions[node](state)


def main():
    amble(0, Node.PROMPT_NUMBER, direct)


if __name__ == "__main__":
    main()
