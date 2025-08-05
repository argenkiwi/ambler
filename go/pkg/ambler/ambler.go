package ambler

import (
	"github.com/argenkiwi/ambler/go/pkg/lead"
)

func Amble[S any](state S, next lead.Lead, follow func(lead.Lead, S) (S, lead.Lead, error)) (S, error) {
	currentState, nextLead, err := follow(next, state)
	if err != nil {
		return currentState, err
	}

	if nextLead == lead.None {
		return currentState, nil
	}

	return Amble(currentState, nextLead, follow)
}
