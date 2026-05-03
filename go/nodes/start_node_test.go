package nodes

import (
	"context"
	"testing"

	"github.com/argenkiwi/ambler/go"
)

type mockStartNodeUtils struct {
	input string
	err   error
}

func (m mockStartNodeUtils) Prompt(message string) (string, error) {
	return m.input, m.err
}

func (m mockStartNodeUtils) Error(message string) {}

func TestStartNode(t *testing.T) {
	ctx := context.Background()
	edges := StartNodeEdges[string]{
		OnSuccess: ambler.Ptr("count"),
		OnError:   ambler.Ptr("start"),
	}

	t.Run("empty input defaults to 0", func(t *testing.T) {
		utils := mockStartNodeUtils{input: ""}
		node := CreateStartNode[mockState, string](edges, utils)
		state := mockState{count: 10}

		next, err := node(ctx, state, "start")
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if next.NodeID == nil || *next.NodeID != "count" {
			t.Errorf("expected count, got %v", next.NodeID)
		}
		if next.State.GetCount() != 0 {
			t.Errorf("expected 0, got %d", next.State.GetCount())
		}
	})

	t.Run("valid input updates count", func(t *testing.T) {
		utils := mockStartNodeUtils{input: "42"}
		node := CreateStartNode[mockState, string](edges, utils)
		state := mockState{count: 10}

		next, err := node(ctx, state, "start")
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if next.NodeID == nil || *next.NodeID != "count" {
			t.Errorf("expected count, got %v", next.NodeID)
		}
		if next.State.GetCount() != 42 {
			t.Errorf("expected 42, got %d", next.State.GetCount())
		}
	})

	t.Run("invalid input transitions to error", func(t *testing.T) {
		utils := mockStartNodeUtils{input: "invalid"}
		node := CreateStartNode[mockState, string](edges, utils)
		state := mockState{count: 10}

		next, err := node(ctx, state, "start")
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if next.NodeID == nil || *next.NodeID != "start" {
			t.Errorf("expected start, got %v", next.NodeID)
		}
		if next.State.GetCount() != 10 {
			t.Errorf("expected 10, got %d", next.State.GetCount())
		}
	})
}
