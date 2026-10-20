// Package start prompts for a starting number.
package start

import (
	"bufio"
	"context"
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/argenkiwi/ambler/go"
	"github.com/argenkiwi/ambler/go/nodes"
)

// Edges defines the possible transitions from the node.
type Edges[K comparable] struct {
	OnSuccess *K
	OnError   *K
}

// Utils defines the side-effectful operations of the node.
type Utils interface {
	// ReadLine prints message and reads a line; ok is false if no input could be read.
	ReadLine(message string) (line string, ok bool)
	Print(message string)
}

var stdin = bufio.NewReader(os.Stdin)

// DefaultUtils is the production implementation of Utils.
type DefaultUtils struct{}

func (DefaultUtils) ReadLine(message string) (string, bool) {
	fmt.Print(message)
	input, err := stdin.ReadString('\n')
	if err != nil && input == "" {
		return "", false
	}
	return strings.TrimRight(input, "\r\n"), true
}

func (DefaultUtils) Print(message string) {
	fmt.Println(message)
}

// Factory creates the node. A nil utils uses DefaultUtils.
func Factory[S nodes.HasCount[S], K comparable](edges Edges[K], utils Utils) ambler.Node[S, K] {
	if utils == nil {
		utils = DefaultUtils{}
	}

	return func(ctx context.Context, state S) (ambler.Next[S, K], error) {
		input, ok := utils.ReadLine("Enter a starting number: ")

		if !ok || input == "" {
			return ambler.Next[S, K]{NodeID: edges.OnSuccess, State: state.WithCount(0)}, nil
		}

		n, err := strconv.Atoi(input)
		if err != nil {
			utils.Print("Error: Invalid input. Please enter a number.")
			return ambler.Next[S, K]{NodeID: edges.OnError, State: state}, nil
		}

		return ambler.Next[S, K]{NodeID: edges.OnSuccess, State: state.WithCount(n)}, nil
	}
}
