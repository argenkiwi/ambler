package ambler

import "context"

func Resolve[S any, A any, D any](result struct {
	State  S
	Action A
}, direct func(A) D) (S, D) {
	return result.State, direct(result.Action)
}

func Amble[S any, E any](ctx context.Context, state S, edge E, follow func(context.Context, S, E) (S, E, error)) (S, E, error) {
	var zero E
	var err error
	for edge != zero {
		state, edge, err = follow(ctx, state, edge)
		if err != nil {
			return state, zero, err
		}
	}
	return state, zero, nil
}