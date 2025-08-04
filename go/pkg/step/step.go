package step

import "github.com/argenkiwi/ambler/go/pkg/lead"

type Step[S any] func(S) (S, lead.Lead, error)