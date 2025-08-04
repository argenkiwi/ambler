<?php

require_once __DIR__ . '/../ambler.php';

class Stop implements Step {
    public function resolve($state) {
        echo "Final count: {$state['count']}\n";
        return [$state, null];
    }
}

