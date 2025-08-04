package step

import (
	"fmt"
	"math/rand"
	"time"

	"github.com/argenkiwi/ambler/go/pkg/lead"
)

func Count(state int) (int, lead.Lead, error) {
	fmt.Printf("Current count: %d\n", state)
	time.Sleep(1 * time.Second)
	state++
	if rand.Float64() > 0.5 {
		return state, lead.Count, nil
	} else {
		return state, lead.Stop, nil
	}
}
