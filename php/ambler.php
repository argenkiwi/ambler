<?php

interface Step {
    public function resolve($state);
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
