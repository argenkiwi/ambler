package nodes

import (
	"context"
	"testing"
	"time"

	"github.com/argenkiwi/ambler/go"
)

type mockCountNodeUtils struct {
	randomVal float64
}

func (m mockCountNodeUtils) Log(message string) {}

func (m mockCountNodeUtils) Sleep(ctx context.Context, duration time.Duration) error {
	return nil
}

func (m mockCountNodeUtils) Random() float64 {
	return m.randomVal
}

func TestCountNode(t *testing.T) {
	ctx := context.Background()
	edges := CountNodeEdges[string]{
		OnCount: ambler.Ptr("count"),
		OnStop:  ambler.Ptr("stop"),
	}

	t.Run("increments count and continues when random <= 0.7", func(t *testing.T) {
		utils := mockCountNodeUtils{randomVal: 0.5}
		node := CreateCountNode[mockState, string](edges, utils)
		state := mockState{count: 5}

		next, err := node(ctx, state, "count")
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if next.NodeID == nil || *next.NodeID != "count" {
			t.Errorf("expected count, got %v", next.NodeID)
		}
		if next.State.GetCount() != 6 {
			t.Errorf("expected 6, got %d", next.State.GetCount())
		}
	})

	t.Run("increments count and stops when random > 0.7", func(t *testing.T) {
		utils := mockCountNodeUtils{randomVal: 0.8}
		node := CreateCountNode[mockState, string](edges, utils)
		state := mockState{count: 5}

		next, err := node(ctx, state, "count")
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if next.NodeID == nil || *next.NodeID != "stop" {
			t.Errorf("expected stop, got %v", next.NodeID)
		}
		if next.State.GetCount() != 6 {
			t.Errorf("expected 6, got %d", next.State.GetCount())
		}
	})
}
