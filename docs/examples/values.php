<?php
$df = new DataFrame([
    'n' => [1, null],
    'x' => [1.5, 2.0],
    'ok' => [true, false],
    'when' => [new DateTimeImmutable('2024-05-01 12:30:00'), null],
    'tags' => [['a', 'b'], []],
]);

print_r($df->schema());
var_dump($df->toDicts()[0]);
