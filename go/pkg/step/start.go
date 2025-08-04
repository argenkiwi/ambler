package step

import (
	"bufio"
	"fmt"
	"strconv"
	"strings"

	"github.com/argenkiwi/ambler/go/pkg/lead"
)

func Start(state int, reader *bufio.Reader) (int, lead.Lead, error) {
	fmt.Print("Enter a starting number (or press Enter for default 0): ")
	input, err := reader.ReadString('\n')
	if err != nil {
		return state, lead.Start, err
	}
	input = strings.TrimSpace(input)

	if input == "" {
		fmt.Printf("Starting count from default: %d\n", state)
		return state, lead.Count, nil
	}

	number, err := strconv.Atoi(input)
	if err != nil {
		fmt.Println("Invalid input. Please, try again.")
		return state, lead.Start, nil
	}

	fmt.Printf("Starting count from: %d\n", number)
	return number, lead.Count, nil
}
