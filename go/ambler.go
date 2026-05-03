package ambler

import (
	"context"
	"fmt"
)

// Next represents the result of a node execution.
type Next[S any, K comparable] struct {
	NodeID *K
	State  S
}

// Node is a function that represents a node in the state machine.
// S is the state type, K is the node identifier type.
type Node[S any, K comparable] func(ctx context.Context, state S, key K) (Next[S, K], error)

// AmblerFunc is the type returned by the Ambler function.
type AmblerFunc[S any, K comparable] func(ctx context.Context, nodeID K, state S) (Next[S, K], error)

// Ambler creates a single-step executor for a node registry.
func Ambler[S any, K comparable](nodes map[K]Node[S, K]) AmblerFunc[S, K] {
	return func(ctx context.Context, nodeID K, state S) (Next[S, K], error) {
		node, ok := nodes[nodeID]
		if !ok {
			return Next[S, K]{}, fmt.Errorf("node not found: %v", nodeID)
		}

		return node(ctx, state, nodeID)
	}
}

// Ptr is a helper to return a pointer to a value, useful for NodeID in Next.
func Ptr[T any](v T) *T {
	return &v
}
