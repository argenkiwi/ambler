package start

import (
	"context"
	"testing"

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
	input string
}

func (m mockUtils) ReadLine(message string) (string, bool) { return m.input, true }

func (m mockUtils) Print(message string) {}

func TestStartNode(t *testing.T) {
	edges := Edges[string]{OnSuccess: ambler.Ptr("NEXT"), OnError: ambler.Ptr("ERROR")}

	tests := []struct {
		name      string
		input     string
		state     mockState
		wantNode  string
		wantCount int
	}{
		{"transitions to onSuccess with 0 if input is empty", "", mockState{0}, "NEXT", 0},
		{"transitions to onSuccess with input number", "42", mockState{0}, "NEXT", 42},
		{"transitions to onError if input is invalid", "abc", mockState{123}, "ERROR", 123},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			node := Factory[mockState](edges, mockUtils{input: tt.input})

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
