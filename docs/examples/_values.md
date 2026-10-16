```php
$df = new DataFrame([
    'n' => [1, null],
    'x' => [1.5, 2.0],
    'ok' => [true, false],
    'when' => [new DateTimeImmutable('2024-05-01 12:30:00'), null],
    'tags' => [['a', 'b'], []],
]);

print_r($df->schema());
var_dump($df->toDicts()[0]);
```

```{.text .output}
Array
(
    [n] => Int64
    [x] => Float64
    [ok] => Boolean
    [when] => Datetime(us)
    [tags] => List(String)
)
array(5) {
  ["n"]=>
  int(1)
  ["x"]=>
  float(1.5)
  ["ok"]=>
  bool(true)
  ["when"]=>
  object(DateTimeImmutable)#3 (3) {
    ["date"]=>
    string(26) "2024-05-01 12:30:00.000000"
    ["timezone_type"]=>
    int(3)
    ["timezone"]=>
    string(3) "UTC"
  }
  ["tags"]=>
  array(2) {
    [0]=>
    string(1) "a"
    [1]=>
    string(1) "b"
  }
}
```
