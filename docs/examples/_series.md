```php
$s = new Series('units', [3, 1, null, 7, 4]);

echo $s, "\n";
var_dump($s->sum(), $s->mean(), $s->nullCount(), $s->dtype());
print_r($s->fillNull(0)->sort(descending: true)->toList());
print_r($s->gt(2)->toList());
```

```{.text .output}
shape: (5,)
Series: 'units' [i64]
[
	3
	1
	null
	7
	4
]
int(15)
float(3.75)
int(1)
string(5) "Int64"
Array
(
    [0] => 7
    [1] => 4
    [2] => 3
    [3] => 1
    [4] => 0
)
Array
(
    [0] => 1
    [1] => 
    [2] => 
    [3] => 1
    [4] => 1
)
```
