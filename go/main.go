package main

import (
	"bufio"
	"fmt"
	"math/rand"
	"os"
	"strconv"
	"strings"
	"time"

	"ambler"
)

func start(state int) (*ambler.Next, error) {
	reader := bufio.NewReader(os.Stdin)
	fmt.Print("Enter a starting number (or press Enter for default): ")
	input, err := reader.ReadString('\n')
	if err != nil {
		return nil, err
	}
	input = strings.TrimSpace(input)

	if input == "" {
		fmt.Println("Using default starting number.")
		return &ambler.Next{Run: func() (*ambler.Next, error) { return count(state) }}, nil
	}

	number, err := strconv.Atoi(input)
	if err != nil {
		fmt.Println("Invalid number, please try again.")
		return &ambler.Next{Run: func() (*ambler.Next, error) { return start(state) }}, nil
	}

	return &ambler.Next{Run: func() (*ambler.Next, error) { return count(number) }}, nil
}

func count(state int) (*ambler.Next, error) {
	fmt.Printf("Count: %d\n", state)
	time.Sleep(1 * time.Second)
	newState := state + 1
	if rand.Float64() > 0.5 {
		return &ambler.Next{Run: func() (*ambler.Next, error) { return count(newState) }}, nil
	} else {
		return &ambler.Next{Run: func() (*ambler.Next, error) { return stop(newState) }}, nil
	}
}

func stop(state int) (*ambler.Next, error) {
	fmt.Printf("Stopping count at %d.\n", state)
	return nil, nil
}

func main() {
	ambler.Amble(start, 0)
}
