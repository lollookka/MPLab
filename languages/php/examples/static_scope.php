<?php
function nextTicket(): int
{
    static $ticket = 0;
    return ++$ticket;
}

echo "Ticket ", nextTicket(), PHP_EOL;
echo "Ticket ", nextTicket(), PHP_EOL;
