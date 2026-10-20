// Package count prints and increments the count, then randomly continues or stops.
package count

import (
	"context"
	"fmt"
	"math/rand"
	"time"

	"github.com/argenkiwi/ambler/go"
	"github.com/argenkiwi/ambler/go/nodes"
)

// Edges defines the possible transitions from the node.
type Edges[K comparable] struct {
	OnCount *K
	OnStop  *K
}

// Utils defines the side-effectful operations of the node.
type Utils interface {
	Print(message string)
	Sleep(ctx context.Context, duration time.Duration) error
	Random() float64
}

// DefaultUtils is the production implementation of Utils.
type DefaultUtils struct{}

func (DefaultUtils) Print(message string) {
	fmt.Println(message)
}

func (DefaultUtils) Sleep(ctx context.Context, duration time.Duration) error {
	select {
	case <-time.After(duration):
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

func (DefaultUtils) Random() float64 {
	return rand.Float64()
}

// Factory creates the node. A nil utils uses DefaultUtils.
func Factory[S nodes.HasCount[S], K comparable](edges Edges[K], utils Utils) ambler.Node[S, K] {
	if utils == nil {
		utils = DefaultUtils{}
	}

	return func(ctx context.Context, state S) (ambler.Next[S, K], error) {
		utils.Print(fmt.Sprintf("Current count: %d", state.GetCount()))
		if err := utils.Sleep(ctx, time.Second); err != nil {
			return ambler.Next[S, K]{}, err
		}
		nextState := state.WithCount(state.GetCount() + 1)

		if utils.Random() > 0.5 {
			return ambler.Next[S, K]{NodeID: edges.OnCount, State: nextState}, nil
		}
		return ambler.Next[S, K]{NodeID: edges.OnStop, State: nextState}, nil
	}
}
