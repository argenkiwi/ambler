// Package nodes holds interfaces shared by the node packages beneath it.
package nodes

// HasCount is a generic interface for state that has a count field.
type HasCount[S any] interface {
	GetCount() int
	WithCount(count int) S
}
