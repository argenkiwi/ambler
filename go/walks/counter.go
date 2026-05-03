package main

import (
	"context"
	"fmt"

	"github.com/argenkiwi/ambler/go"
	"github.com/argenkiwi/ambler/go/nodes"
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
	StartID NodeID = "start"
	CountID NodeID = "count"
	StopID  NodeID = "stop"
)

func main() {
	// Wire the nodes together.
	nodesMap := map[NodeID]ambler.Node[State, NodeID]{
		StartID: nodes.CreateStartNode[State, NodeID](nodes.StartNodeEdges[NodeID]{
			OnSuccess: ambler.Ptr(CountID),
			OnError:   ambler.Ptr(StartID),
		}, nil),
		CountID: nodes.CreateCountNode[State, NodeID](nodes.CountNodeEdges[NodeID]{
			OnCount: ambler.Ptr(CountID),
			OnStop:  ambler.Ptr(StopID),
		}, nil),
		StopID: nodes.CreateStopNode[State, NodeID](nodes.StopNodeEdges[NodeID]{
			OnDone: nil, // Terminal node
		}, nil),
	}

	// Create the Ambler executor.
	amble := ambler.Ambler(nodesMap)

	ctx := context.Background()
	var nodeID *NodeID = ambler.Ptr(StartID)
	state := State{Count: 0}

	// Run the state machine.
	for nodeID != nil {
		next, err := amble(ctx, *nodeID, state)
		if err != nil {
			fmt.Printf("Error during execution: %v\n", err)
			break
		}
		nodeID = next.NodeID
		state = next.State
	}
}
