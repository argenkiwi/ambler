package main

import (
	"context"
	"fmt"

	"github.com/argenkiwi/ambler/go"
	"github.com/argenkiwi/ambler/go/nodes/count"
	"github.com/argenkiwi/ambler/go/nodes/start"
	"github.com/argenkiwi/ambler/go/nodes/stop"
)

// State implements nodes.HasCount for the counter walk.
type State struct {
	Count int
}

func (s State) GetCount() int {
	return s.Count
}

func (s State) WithCount(count int) State {
	s.Count = count
	return s
}

// NodeID is the type for node identifiers in this walk.
type NodeID string

const (
	Start NodeID = "START"
	Count NodeID = "COUNT"
	Stop  NodeID = "STOP"
)

var amble = ambler.Ambler(map[NodeID]func() ambler.Node[State, NodeID]{
	Start: func() ambler.Node[State, NodeID] {
		return start.Factory[State](start.Edges[NodeID]{OnSuccess: ambler.Ptr(Count), OnError: ambler.Ptr(Start)}, nil)
	},
	Count: func() ambler.Node[State, NodeID] {
		return count.Factory[State](count.Edges[NodeID]{OnCount: ambler.Ptr(Count), OnStop: ambler.Ptr(Stop)}, nil)
	},
	Stop: func() ambler.Node[State, NodeID] {
		return stop.Factory[State](stop.Edges[NodeID]{OnDone: nil}, nil)
	},
})

func main() {
	ctx := context.Background()
	nodeID := ambler.Ptr(Start)
	state := State{Count: 0}

	for nodeID != nil {
		next, err := amble(ctx, *nodeID, state)
		if err != nil {
			fmt.Printf("Error during execution: %v\n", err)
			break
		}
		nodeID, state = next.NodeID, next.State
	}
}
