<?php

require_once __DIR__ . '/../lead.php';

function countStep($state) {
    echo "Current count: $state\n";
    sleep(1);
    $state++;

    if (rand(0, 1) < 0.5) {
        return [$state, Lead::COUNT];
    } else {
        return [$state, Lead::STOP];
    }
}

