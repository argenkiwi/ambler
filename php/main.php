<?php

require_once 'ambler.php';
require_once 'step/start.php';
require_once 'step/count.php';
require_once 'step/stop.php';

$initialState = ['count' => 0];

amble($initialState, 'start', function ($lead) {
    switch ($lead) {
        case 'start':
            return new Start();
        case 'count':
            return new CountStep();
        case 'stop':
            return new Stop();
    }
});
