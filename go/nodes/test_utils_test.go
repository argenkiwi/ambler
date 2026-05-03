package nodes

type mockState struct {
	count int
}

func (s mockState) GetCount() int {
	return s.count
}

func (s mockState) WithCount(count int) mockState {
	s.count = count
	return s
}
