package main

import (
	"bufio"
	"fmt"
	"os"

	"github.com/argenkiwi/ambler/go/pkg/ambler"
	"github.com/argenkiwi/ambler/go/pkg/lead"
	"github.com/argenkiwi/ambler/go/pkg/step"
)

func main() {
	initialState := 0
	initialLead := lead.Start
	reader := bufio.NewReader(os.Stdin)

	_, err := ambler.Amble(initialState, initialLead, func(l lead.Lead, s int) (int, lead.Lead, error) {
		switch l {
		case lead.Start:
			return step.Start(s, reader)
		case lead.Count:
			return step.Count(s)
		case lead.Stop:
			return step.Stop(s)
		default:
			panic(fmt.Sprintf("Unknown lead: %v", l))
		}
	})
	if err != nil {
		panic(err)
	}
}
