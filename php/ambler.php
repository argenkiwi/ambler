<?php

function amble($state, $lead, callable $follow) {
    $currentLead = $lead;
    $currentState = $state;

    while ($currentLead !== null) {
        list($currentState, $currentLead) = $follow($currentLead, $currentState);
    }

    return $currentState;
}
