<?php
$wide = new DataFrame(['id' => [1, 2], 'q1' => [10, 20], 'q2' => [11, 21]]);

$long = $wide->unpivot(index: 'id', variableName: 'quarter', valueName: 'sales');
echo $long, "\n";
echo $long->pivot('quarter', index: 'id', values: 'sales'), "\n";
echo concat([$wide, new DataFrame(['id' => [3], 'q1' => [30], 'q2' => [31]])]), "\n";
echo $wide->describe();
