package ambler

type Next struct {
	Run func() (*Next, error)
}

func Amble(initial *Next) error {
	next := initial
	var err error
	for next != nil {
		next, err = next.Run()
		if err != nil {
			return err
		}
	}
	return nil
}

func AmbleFromFunc(initial func() (*Next, error)) error {
	return Amble(&Next{Run: initial})
}
