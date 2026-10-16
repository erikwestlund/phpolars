<?php

declare(strict_types=1);

use Phibble\Contracts\Table;
use Phpolars\Phibble\PolarsTable;

// phibble's shared Expr cases and dplyr fixtures, from the installed package.
require_once __DIR__.'/../vendor/erikwestlund/phibble/tests/Support/ExprCases.php';

function pt(Table $table): PolarsTable
{
    return PolarsTable::fromTable($table);
}

/** Run $fn and fail if any verb fell back to an in-memory Frame. */
function native(Closure $fn): mixed
{
    $before = PolarsTable::$fallbacks;
    $result = $fn();
    expect(PolarsTable::$fallbacks)->toBe($before, 'the verb fell back to a phibble Frame');

    return $result;
}

/** Run $fn with warnings collected instead of thrown. */
function quietly(Closure $fn): array
{
    $warnings = [];
    set_error_handler(function (int $level, string $message) use (&$warnings) {
        $warnings[] = $message;

        return true;
    });
    try {
        $result = $fn();
    } finally {
        restore_error_handler();
    }

    return [$result, $warnings];
}
