<?php
// php -d extension=... scripts/docs-reference.php: writes docs/reference/_api-<name>.md,
// signature lists generated from the extension's reflection data.

$out = dirname(__DIR__).'/docs/reference';

function sig(ReflectionFunctionAbstract $f): string
{
    $params = array_map(function (ReflectionParameter $p) {
        $s = ($p->getType() ? $p->getType().' ' : '').($p->isVariadic() ? '...' : '').'$'.$p->getName();

        return $p->isOptional() && ! $p->isVariadic() ? $s.' = null' : $s;
    }, $f->getParameters());
    $ret = $f->getReturnType() ? ': '.$f->getReturnType() : '';

    return $f->getShortName().'('.implode(', ', $params).')'.$ret;
}

function methods(string $class): string
{
    $r = new ReflectionClass($class);
    $lines = [];
    foreach ($r->getMethods() as $m) {
        if ($m->getDeclaringClass()->getName() !== $class || $m->getName() === '__construct') {
            continue;
        }
        $lines[] = ($m->isStatic() ? 'static ' : '').sig($m);
    }
    sort($lines);

    return "```php\n".implode("\n", $lines)."\n```\n";
}

$pages = [
    'dataframe' => ['Phpolars\\DataFrame', 'Phpolars\\LazyFrame', 'Phpolars\\GroupBy', 'Phpolars\\LazyGroupBy'],
    'expressions' => ['Phpolars\\Expr', 'Phpolars\\When', 'Phpolars\\Then', 'Phpolars\\StrNameSpace', 'Phpolars\\DtNameSpace', 'Phpolars\\ListNameSpace', 'Phpolars\\StructNameSpace', 'Phpolars\\NameNameSpace'],
    'series' => ['Phpolars\\Series'],
];
foreach ($pages as $page => $classes) {
    $md = '';
    foreach ($classes as $class) {
        $md .= "### `{$class}`\n\n".methods($class)."\n";
    }
    file_put_contents("{$out}/_api-{$page}.md", $md);
}

$functions = array_filter(get_extension_funcs('php_polars'), fn ($f) => str_starts_with(strtolower($f), 'phpolars\\'));
$io = ['read_', 'scan_'];
$expr = [];
$read = [];
foreach ($functions as $f) {
    $r = new ReflectionFunction($f);
    $line = sig($r);
    if (array_any($io, fn ($p) => str_starts_with($r->getShortName(), $p))) {
        $read[] = $line;
    } else {
        $expr[] = $line;
    }
}
sort($expr);
sort($read);
file_put_contents("{$out}/_api-functions.md", "```php\n".implode("\n", $expr)."\n```\n");
file_put_contents("{$out}/_api-io.md", "```php\n".implode("\n", $read)."\n```\n");
echo "wrote reference signatures\n";
