package main

import (
	"context"
	"fmt"
	"math/rand"
	"time"

	"/home/argenkiwi/Code/ambler/go"
)

type Node int

const (
	NodeStart Node = iota
	NodeStep
	NodeStop
)

func start(state int) (int, any) {
	fmt.Println("Let's count...")
	return state, nil
}

func step(state int) (int, bool) {
	count := state + 1
	fmt.Printf("...%d...\n", count)
	return count, rand.Intn(2) == 0
}

func stop(state int) (int, any) {
	fmt.Println("...stop.")
	return state, nil
}

func main() {
	rand.Seed(time.Now().UnixNano())
	ambler.Amble(context.Background(), 0, NodeStart, func(ctx context.Context, state int, node Node) (int, Node, error) {
		switch node {
		case NodeStart:
			return ambler.Resolve(struct{State int; Action any}{start(state)}, func(_ any) Node { return NodeStep })
		case NodeStep:
			return ambler.Resolve(struct{State int; Action bool}{step(state)}, func(shouldContinue bool) Node {
				if shouldContinue {
					return NodeStep
				} else {
					return NodeStop
				}
			})
		case NodeStop:
			return ambler.Resolve(struct{State int; Action any}{stop(state)}, func(_ any) Node { return -1 })
		}
		return state, -1, nil
	})
}