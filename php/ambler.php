<?php

interface Step {
    public function resolve($state);
}

class NextStep implements Step {
    private $delegate;

    public function __construct(callable $delegate) {
        $this->delegate = $delegate;
    }

    public function resolve($state) {
        return call_user_func($this->delegate, $state);
    }
}

function amble($state, $lead, callable $follow) {
    $currentLead = $lead;
    $currentState = $state;

    while ($currentLead !== null) {
        $step = $follow($currentLead);
        list($currentState, $currentLead) = $step->resolve($currentState);
    }

    return $currentState;
}
