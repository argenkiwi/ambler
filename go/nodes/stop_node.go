package nodes

import (
	"context"
	"fmt"

	"github.com/argenkiwi/ambler/go"
)

// StopNodeUtils defines the side-effectful operations for StopNode.
type StopNodeUtils interface {
	Log(message string)
}

// DefaultStopNodeUtils is the production implementation of StopNodeUtils.
type DefaultStopNodeUtils struct{}

func (d DefaultStopNodeUtils) Log(message string) {
	fmt.Println(message)
}

// StopNodeEdges defines the possible transitions from StopNode.
type StopNodeEdges[K comparable] struct {
	OnDone *K
}

// CreateStopNode creates a new StopNode.
func CreateStopNode[S HasCount[S], K comparable](edges StopNodeEdges[K], utils StopNodeUtils) ambler.Node[S, K] {
	if utils == nil {
		utils = DefaultStopNodeUtils{}
	}

	return func(ctx context.Context, state S, key K) (ambler.Next[S, K], error) {
		utils.Log(fmt.Sprintf("Final count: %d", state.GetCount()))
		utils.Log("Counting process terminated.")
		return ambler.Next[S, K]{NodeID: edges.OnDone, State: state}, nil
	}
}
