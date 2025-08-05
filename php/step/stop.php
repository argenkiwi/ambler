<?php

function stop($state) {
    echo "Final count: $state\n";
    return [$state, null];
}

