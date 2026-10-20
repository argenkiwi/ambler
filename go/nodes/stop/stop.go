// Package stop prints the final count and terminates.
package stop

import (
	"context"
	"fmt"

	"github.com/argenkiwi/ambler/go"
	"github.com/argenkiwi/ambler/go/nodes"
)

// Edges defines the possible transitions from the node.
type Edges[K comparable] struct {
	OnDone *K
}

// Utils defines the side-effectful operations of the node.
type Utils interface {
	Print(message string)
}

// DefaultUtils is the production implementation of Utils.
type DefaultUtils struct{}

func (DefaultUtils) Print(message string) {
	fmt.Println(message)
}

// Factory creates the node. A nil utils uses DefaultUtils.
func Factory[S nodes.HasCount[S], K comparable](edges Edges[K], utils Utils) ambler.Node[S, K] {
	if utils == nil {
		utils = DefaultUtils{}
	}

	return func(ctx context.Context, state S) (ambler.Next[S, K], error) {
		utils.Print(fmt.Sprintf("Final count: %d", state.GetCount()))
		return ambler.Next[S, K]{NodeID: edges.OnDone, State: state}, nil
	}
}
