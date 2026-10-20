package ambler

import (
	"context"
	"fmt"
)

// Next is the result of a node's execution: the next node ID (or nil to stop) and the updated state.
type Next[S any, K comparable] struct {
	NodeID *K
	State  S
}

// Node is a processing step in the state machine.
// It receives the current state and returns a Next.
type Node[S any, K comparable] func(ctx context.Context, state S) (Next[S, K], error)

// AmblerFunc is the runner returned by Ambler.
type AmblerFunc[S any, K comparable] func(ctx context.Context, nodeID K, state S) (Next[S, K], error)

// Ambler creates a state machine runner from a map of node factories.
//
// Each factory is called once on first use; the resulting node is cached for subsequent calls.
// The runner returns an error if nodeID is not present in nodes.
func Ambler[S any, K comparable](nodes map[K]func() Node[S, K]) AmblerFunc[S, K] {
	loaded := map[K]Node[S, K]{}
	return func(ctx context.Context, nodeID K, state S) (Next[S, K], error) {
		factory, ok := nodes[nodeID]
		if !ok {
			return Next[S, K]{}, fmt.Errorf("node not found: %v", nodeID)
		}
		node, ok := loaded[nodeID]
		if !ok {
			node = factory()
			loaded[nodeID] = node
		}
		return node(ctx, state)
	}
}

// Ptr is a helper to return a pointer to a value, useful for edges and NodeID in Next.
func Ptr[T any](v T) *T {
	return &v
}
