<?php
// Minimal assertion helpers shared by the test files; exits non-zero on the first failure.

function check(string $name, mixed $actual, mixed $expected): void {
    if ($actual != $expected || gettype($actual) !== gettype($expected)) {
        fwrite(STDERR, "FAIL $name\nexpected: " . var_export($expected, true) . "\nactual:   " . var_export($actual, true) . "\n");
        exit(1);
    }
    $GLOBALS['passed'] = ($GLOBALS['passed'] ?? 0) + 1;
}

function same(string $name, mixed $actual, mixed $expected): void {
    if ($actual !== $expected) {
        fwrite(STDERR, "FAIL $name\nexpected: " . var_export($expected, true) . "\nactual:   " . var_export($actual, true) . "\n");
        exit(1);
    }
    $GLOBALS['passed'] = ($GLOBALS['passed'] ?? 0) + 1;
}

function throws(string $name, callable $f, string $class, string $message = ''): void {
    try {
        $f();
    } catch (Throwable $e) {
        if (!$e instanceof $class || !str_contains($e->getMessage(), $message)) {
            fwrite(STDERR, "FAIL $name\nexpected $class containing '$message'\ngot " . get_class($e) . ': ' . $e->getMessage() . "\n");
            exit(1);
        }
        $GLOBALS['passed'] = ($GLOBALS['passed'] ?? 0) + 1;
        return;
    }
    fwrite(STDERR, "FAIL $name: no exception\n");
    exit(1);
}

register_shutdown_function(function () {
    if (error_get_last() === null) {
        printf("%s: %d checks passed\n", basename($_SERVER['SCRIPT_NAME']), $GLOBALS['passed'] ?? 0);
    }
});
