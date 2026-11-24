<?php
function addFee(int $price, int $fee = 2): int
{
    $price += $fee;
    return $price;
}

$price = 10;
echo "With fee: ", addFee($price), PHP_EOL;
echo "Original: $price", PHP_EOL;
