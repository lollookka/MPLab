<?php
$prices = ["Pen" => 2, "Notebook" => 5, "Folder" => 3];
$total = 0;

foreach ($prices as $price) {
    $total += $price;
}

echo "Total: $total", PHP_EOL;
