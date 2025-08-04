package step

import (
	"fmt"

	"github.com/argenkiwi/ambler/go/pkg/lead"
)

func Stop(state int) (int, lead.Lead, error) {
	fmt.Printf("Final count: %d\n", state)
	return state, -1, nil
}
