package main

import (
	"bufio"
	"fmt"
	"math/rand"
	"os"
	"strconv"
	"strings"

	"ambler"
)

func promptForNumber() (int, error) {
	reader := bufio.NewReader(os.Stdin)
	for {
		fmt.Print("Enter a starting number: ")
		input, err := reader.ReadString('\n')
		if err != nil {
			return 0, err
		}
		number, err := strconv.Atoi(strings.TrimSpace(input))
		if err == nil {
			return number, nil
		}
		fmt.Println("Invalid number, please try again.")
	}
}

func promptNumberNode(state int) (*ambler.Next, error) {
	number, err := promptForNumber()
	if err != nil {
		return nil, err
	}
	return &ambler.Next{Run: func() (*ambler.Next, error) { return startNode(number) }}, nil
}

func startNode(state int) (*ambler.Next, error) {
	fmt.Printf("Starting count from %d\n", state)
	return &ambler.Next{Run: func() (*ambler.Next, error) { return stepNode(state) }}, nil
}

func stepNode(state int) (*ambler.Next, error) {
	newState := state + 1
	fmt.Printf("Count: %d\n", newState)
	if rand.Float64() > 0.5 {
		return &ambler.Next{Run: func() (*ambler.Next, error) { return stepNode(newState) }}, nil
	} else {
		return &ambler.Next{Run: func() (*ambler.Next, error) { return stopNode(newState) }}, nil
	}
}

func stopNode(state int) (*ambler.Next, error) {
	fmt.Println("Stopping count.")
	return nil, nil
}

func main() {
	ambler.AmbleFromFunc(func() (*ambler.Next, error) { return promptNumberNode(0) })
}
