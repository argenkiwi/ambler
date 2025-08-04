<?php

require_once __DIR__ . '/../ambler.php';

class Start implements Step {
    public function resolve($state) {
        while (true) {
            echo "Enter a starting number (or press Enter for default 0): ";
            $userInput = trim(fgets(STDIN));

            if (empty($userInput)) {
                $state['count'] = 0;
                echo "Starting count from default: {$state['count']}\n";
                return [$state, 'count'];
            }

            if (is_numeric($userInput)) {
                $state['count'] = (int)$userInput;
                echo "Starting count from: {$state['count']}\n";
                return [$state, 'count'];
            } else {
                echo "Invalid input. Please enter a valid number or press Enter.\n";
            }
        }
    }
}

