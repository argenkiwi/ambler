package ambler

import (
	"github.com/argenkiwi/ambler/go/pkg/lead"
)

func Amble[S any](state S, next lead.Lead, follow func(lead.Lead) func(S) (S, lead.Lead, error)) (S, error) {
	resolve := follow(next)
	currentState, nextLead, err := resolve(state)
	if err != nil {
		return currentState, err
	}

	if nextLead == -1 {
		return currentState, nil
	}

	return Amble(currentState, nextLead, follow)
}
