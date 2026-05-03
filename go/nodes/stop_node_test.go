package nodes

import (
	"context"
	"testing"

	"github.com/argenkiwi/ambler/go"
)

type mockStopNodeUtils struct{}

func (m mockStopNodeUtils) Log(message string) {}

func TestStopNode(t *testing.T) {
	ctx := context.Background()
	edges := StopNodeEdges[string]{
		OnDone: nil,
	}

	t.Run("transitions to nil (OnDone)", func(t *testing.T) {
		utils := mockStopNodeUtils{}
		node := CreateStopNode[mockState, string](edges, utils)
		state := mockState{count: 10}

		next, err := node(ctx, state, "stop")
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if next.NodeID != nil {
			t.Errorf("expected nil, got %v", next.NodeID)
		}
		if next.State.GetCount() != 10 {
			t.Errorf("expected 10, got %d", next.State.GetCount())
		}
	})
}
