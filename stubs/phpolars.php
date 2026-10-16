<?php

// IDE and static-analysis stubs for the php_polars extension. Not loaded at runtime.

namespace Phpolars;

/** An eager Polars data frame. Every operation returns a new frame. NA is null. */
final class DataFrame
{
    public static function readCsv(string $path): DataFrame {}

    public static function readParquet(string $path): DataFrame {}

    /** @param array<string, list<int|float|string|bool|null>> $columns */
    public static function fromColumns(array $columns): DataFrame {}

    /** @param list<array<string, int|float|string|bool|null>> $rows */
    public static function fromRows(array $rows): DataFrame {}

    /** Reads any Phibble\Contracts\Table through toColumns(). */
    public static function fromPhibble(object $table): DataFrame {}

    /** @return list<string> */
    public function columns(): array {}

    public function height(): int {}

    public function width(): int {}

    /** @param string|list<string> $names */
    public function select(string|array $names): DataFrame {}

    /** Keep rows where the predicate is true; null counts as false. */
    public function filter(Expr $predicate): DataFrame {}

    /** @param array<string, Expr> $columns Add or replace columns. */
    public function withColumns(array $columns): DataFrame {}

    /**
     * Stable sort; nulls last.
     *
     * @param string|list<string> $by
     * @param bool|list<bool> $descending
     */
    public function sort(string|array $by, bool|array $descending = false): DataFrame {}

    /** @param string|list<string> $keys */
    public function groupBy(string|array $keys): GroupBy {}

    public function head(int $n = 6): DataFrame {}

    /** @return list<array<string, mixed>> */
    public function toRows(): array {}

    /** @return array<string, list<mixed>> */
    public function toColumns(): array {}

    /** Requires erikwestlund/phibble. */
    public function toPhibble(): \Phibble\Frame {}

    public function __toString(): string {}
}

final class GroupBy
{
    /**
     * One row per group, in order of first appearance: keys, then aggregates.
     *
     * @param list<Expr> $aggregates
     */
    public function agg(array $aggregates): DataFrame {}
}

/** A column expression. Comparison and arithmetic operands may be an Expr or a literal. */
final class Expr
{
    public static function col(string $name): Expr {}

    public static function lit(int|float|string|bool|null $value): Expr {}

    public function eq(mixed $other): Expr {}

    public function neq(mixed $other): Expr {}

    public function gt(mixed $other): Expr {}

    public function gte(mixed $other): Expr {}

    public function lt(mixed $other): Expr {}

    public function lte(mixed $other): Expr {}

    public function and(mixed $other): Expr {}

    public function or(mixed $other): Expr {}

    public function not(): Expr {}

    public function isNull(): Expr {}

    public function add(mixed $other): Expr {}

    public function sub(mixed $other): Expr {}

    public function mul(mixed $other): Expr {}

    public function div(mixed $other): Expr {}

    public function mean(): Expr {}

    public function sum(): Expr {}

    /** Non-null values. */
    public function count(): Expr {}

    public function min(): Expr {}

    public function max(): Expr {}

    public function alias(string $name): Expr {}
}

function col(string $name): Expr {}

function lit(int|float|string|bool|null $value): Expr {}

/** Size of the Polars thread pool in this process. */
function threads(): int {}

function version(): string {}
