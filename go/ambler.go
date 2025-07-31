package ambler

type Next struct {
	Run func() (*Next, error)
}

func Amble(initial func(int) (*Next, error), state int) error {
	next, err := initial(state)
	if err != nil {
		return err
	}

	for next != nil {
		next, err = next.Run()
		if err != nil {
			return err
		}
	}
	return nil
}
