<?php
// php -d extension=... scripts/api-compat.php OLD_STUBS
// Checks that every function and method in an earlier release's stubs still
// exists and that its parameters are a prefix of today's, so code written
// against that release (positional or named arguments) keeps working.
$src = file_get_contents($argv[1]);
$ns = ''; $class = null; $bad = 0; $n = 0;
foreach (explode("\n", $src) as $line) {
    if (preg_match('/^namespace ([^;{ ]+)/', $line, $m)) { $ns = $m[1]; $class = null; }
    if (preg_match('/^(\s*)(?:final |abstract )?class (\w+)/', $line, $m)) { $class = $ns . '\\' . $m[2]; $classIndent = strlen($m[1]); }
    if ($class !== null && preg_match('/^(\s*)function /', $line, $fm) && strlen($fm[1]) <= $classIndent) { $class = null; }
    if (preg_match('/^}/', $line)) { $class = null; }
    if (preg_match('/^\s*(?:public )?(?:static )?function (\w+)\((.*)\)/', $line, $m)) {
        preg_match_all('/\$(\w+)/', $m[2], $pm);
        $old = $pm[1];
        try {
            $r = $class ? new ReflectionMethod($class, $m[1]) : new ReflectionFunction($ns . '\\' . $m[1]);
        } catch (Throwable $e) { echo 'MISSING ', $class ? "$class::" : "$ns\\", $m[1], "\n"; $bad++; continue; }
        $new = array_map(fn ($p) => $p->getName(), $r->getParameters());
        $n++;
        if (array_slice($new, 0, count($old)) !== $old) {
            echo 'CHANGED ', $class ? "$class::" : '', $r->getName(), ': (', implode(', ', $old), ') -> (', implode(', ', $new), ")\n";
            $bad++;
        }
    }
}
echo "api-compat: $n signatures checked, $bad problems\n";
exit($bad === 0 ? 0 : 1);
