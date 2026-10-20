package count

import (
	"context"
	"testing"
	"time"

	"github.com/argenkiwi/ambler/go"
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
	random float64
}

func (m mockUtils) Print(message string) {}

func (m mockUtils) Sleep(ctx context.Context, duration time.Duration) error { return nil }

func (m mockUtils) Random() float64 { return m.random }

func TestCountNode(t *testing.T) {
	edges := Edges[string]{OnCount: ambler.Ptr("onCount"), OnStop: ambler.Ptr("onStop")}

	tests := []struct {
		name      string
		random    float64
		state     mockState
		wantNode  string
		wantCount int
	}{
		{"increments count and transitions to onCount if random > 0.5", 0.6, mockState{5}, "onCount", 6},
		{"increments count and transitions to onStop if random <= 0.5", 0.4, mockState{10}, "onStop", 11},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			node := Factory[mockState](edges, mockUtils{random: tt.random})

			next, err := node(context.Background(), tt.state)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			if next.NodeID == nil || *next.NodeID != tt.wantNode {
				t.Errorf("expected %s, got %v", tt.wantNode, next.NodeID)
			}
			if next.State.GetCount() != tt.wantCount {
				t.Errorf("expected %d, got %d", tt.wantCount, next.State.GetCount())
			}
		})
	}
}
