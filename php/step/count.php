<?php

require_once __DIR__ . '/../ambler.php';

class CountStep implements Step {
    public function resolve($state) {
        echo "Current count: {$state['count']}\n";
        sleep(1);
        $state['count']++;

        if (rand(0, 1) < 0.5) {
            return [$state, 'count'];
        } else {
            return [$state, 'stop'];
        }
    }
}

