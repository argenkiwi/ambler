import asyncio
import random
from ambler import amble

# Nodes.
class Node:
    PROMPT_NUMBER = 0
    START = 1
    STEP = 2
    STOP = 3

def prompt_number(state):
    while True:
        try:
            num = int(input("Enter a number to start counting from: "))
            return num, Node.START
        except ValueError:
            print("Invalid input. Please enter an integer.")

def start(state):
    print(f"Let's count from {state}...")
    return state, Node.STEP

async def step(state):
    count = state + 1
    await asyncio.sleep(1)
    print(f"...{count}...")
    return count, Node.STEP if random.choice([True, False]) else Node.STOP

def stop(state):
    print("...stop.")
    return state, None

# Flow.
async def direct(state, node):
    if node == Node.PROMPT_NUMBER:
        return prompt_number(state)
    elif node == Node.START:
        return start(state)
    elif node == Node.STEP:
        return await step(state)
    elif node == Node.STOP:
        return stop(state)

async def main():
    await amble(0, Node.PROMPT_NUMBER, direct)

if __name__ == "__main__":
   asyncio.run(main())
