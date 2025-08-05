<?php

require_once __DIR__ . '/../lead.php';

function start($state) {
    while (true) {
        echo "Enter a starting number (or press Enter for default 0): ";
        $userInput = trim(fgets(STDIN));

        if (empty($userInput)) {
            echo "Starting count from default: $state\n";
            return [$state, Lead::COUNT];
        }

        if (is_numeric($userInput)) {
            $state = (int)$userInput;
            echo "Starting count from: $state\n";
            return [$state, Lead::COUNT];
        } else {
            echo "Invalid input. Please enter a valid number or press Enter.\n";
        }
    }
}

