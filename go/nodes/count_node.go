package nodes

import (
	"context"
	"fmt"
	"math/rand"
	"time"

	"github.com/argenkiwi/ambler/go"
)

// CountNodeUtils defines the side-effectful operations for CountNode.
type CountNodeUtils interface {
	Log(message string)
	Sleep(ctx context.Context, duration time.Duration) error
	Random() float64
}

// DefaultCountNodeUtils is the production implementation of CountNodeUtils.
type DefaultCountNodeUtils struct{}

func (d DefaultCountNodeUtils) Log(message string) {
	fmt.Println(message)
}

func (d DefaultCountNodeUtils) Sleep(ctx context.Context, duration time.Duration) error {
	select {
	case <-time.After(duration):
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

func (d DefaultCountNodeUtils) Random() float64 {
	return rand.Float64()
}

// CountNodeEdges defines the possible transitions from CountNode.
type CountNodeEdges[K comparable] struct {
	OnCount *K
	OnStop  *K
}

// CreateCountNode creates a new CountNode.
func CreateCountNode[S HasCount[S], K comparable](edges CountNodeEdges[K], utils CountNodeUtils) ambler.Node[S, K] {
	if utils == nil {
		utils = DefaultCountNodeUtils{}
	}

	return func(ctx context.Context, state S, key K) (ambler.Next[S, K], error) {
		utils.Log(fmt.Sprintf("Current count: %d", state.GetCount()))

		err := utils.Sleep(ctx, 1*time.Second)
		if err != nil {
			return ambler.Next[S, K]{}, err
		}

		nextState := state.WithCount(state.GetCount() + 1)

		stop := utils.Random() > 0.7

		if stop {
			return ambler.Next[S, K]{NodeID: edges.OnStop, State: nextState}, nil
		}

		return ambler.Next[S, K]{NodeID: edges.OnCount, State: nextState}, nil
	}
}
