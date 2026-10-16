```php
use Phpolars\Exception\{ColumnNotFoundError, SchemaError};

$df = new DataFrame(['a' => [1, 2]]);

try {
    $df->select('b');
} catch (ColumnNotFoundError $e) {
    echo get_class($e), ': ', strtok($e->getMessage(), "\n"), "\n";
}
try {
    new DataFrame(['a' => [1, 'two']]);
} catch (SchemaError $e) {
    echo get_class($e), ': ', $e->getMessage(), "\n";
}
echo $e instanceof Phpolars\PolarsError ? "both extend Phpolars\\PolarsError\n" : '';
```

```{.text .output}
Phpolars\Exception\ColumnNotFoundError: not found: unable to find column "b"; valid columns: ["a"]
Phpolars\Exception\SchemaError: column 'a' mixes incompatible types; pass a dtype or cast first
both extend Phpolars\PolarsError
```
