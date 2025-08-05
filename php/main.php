<?php

require_once 'ambler.php';
require_once 'lead.php';
require_once 'step/start.php';
require_once 'step/count.php';
require_once 'step/stop.php';

$initialState = 0;

amble($initialState, Lead::START, function ($lead, $state) {
    switch ($lead) {
        case Lead::START:
            return start($state);
        case Lead::COUNT:
            return countStep($state);
        case Lead::STOP:
            return stop($state);
    }
});
