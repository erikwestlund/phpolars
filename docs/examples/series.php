<?php
$s = new Series('units', [3, 1, null, 7, 4]);

echo $s, "\n";
var_dump($s->sum(), $s->mean(), $s->nullCount(), $s->dtype());
print_r($s->fillNull(0)->sort(descending: true)->toList());
print_r($s->gt(2)->toList());
