package stop

import (
	"context"
	"testing"
)

type mockState struct {
	count int
}

func (s mockState) GetCount() int { return s.count }

func (s mockState) WithCount(count int) mockState {
	s.count = count
	return s
}

type mockUtils struct {
	message *string
}

func (m mockUtils) Print(message string) { *m.message = message }

func TestStopNode(t *testing.T) {
	var captured string
	node := Factory[mockState](Edges[string]{OnDone: nil}, mockUtils{message: &captured})

	next, err := node(context.Background(), mockState{15})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if next.NodeID != nil {
		t.Errorf("expected nil, got %v", *next.NodeID)
	}
	if next.State.GetCount() != 15 {
		t.Errorf("expected 15, got %d", next.State.GetCount())
	}
	if captured != "Final count: 15" {
		t.Errorf("expected %q, got %q", "Final count: 15", captured)
	}
}
