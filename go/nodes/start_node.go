package nodes

import (
	"bufio"
	"context"
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/argenkiwi/ambler/go"
)

// StartNodeUtils defines the side-effectful operations for StartNode.
type StartNodeUtils interface {
	Prompt(message string) (string, error)
	Error(message string)
}

// DefaultStartNodeUtils is the production implementation of StartNodeUtils.
type DefaultStartNodeUtils struct{}

func (d DefaultStartNodeUtils) Prompt(message string) (string, error) {
	fmt.Print(message)
	reader := bufio.NewReader(os.Stdin)
	input, err := reader.ReadString('\n')
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(input), nil
}

func (d DefaultStartNodeUtils) Error(message string) {
	fmt.Fprintln(os.Stderr, message)
}

// StartNodeEdges defines the possible transitions from StartNode.
type StartNodeEdges[K comparable] struct {
	OnSuccess *K
	OnError   *K
}

// HasCount is a generic interface for state that has a count field.
type HasCount[S any] interface {
	GetCount() int
	WithCount(count int) S
}

// CreateStartNode creates a new StartNode.
func CreateStartNode[S HasCount[S], K comparable](edges StartNodeEdges[K], utils StartNodeUtils) ambler.Node[S, K] {
	if utils == nil {
		utils = DefaultStartNodeUtils{}
	}

	return func(ctx context.Context, state S, key K) (ambler.Next[S, K], error) {
		input, err := utils.Prompt("Enter a starting number (default 0):")
		if err != nil {
			// If we can't read input, we'll treat it as empty input (default 0).
			return ambler.Next[S, K]{
				NodeID: edges.OnSuccess,
				State:  state.WithCount(0),
			}, nil
		}

		if input == "" {
			return ambler.Next[S, K]{
				NodeID: edges.OnSuccess,
				State:  state.WithCount(0),
			}, nil
		}

		count, err := strconv.Atoi(input)
		if err != nil {
			utils.Error("Invalid number entered.")
			return ambler.Next[S, K]{
				NodeID: edges.OnError,
				State:  state,
			}, nil
		}

		return ambler.Next[S, K]{
			NodeID: edges.OnSuccess,
			State:  state.WithCount(count),
		}, nil
	}
}
