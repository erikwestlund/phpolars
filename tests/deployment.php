<?php
// The same assertions through the CLI and a real PHP-FPM FastCGI request.
declare(strict_types=1);

use function Phpolars\col;

try {
    if (! extension_loaded('php_polars')) {
        throw new RuntimeException('php_polars is not enabled');
    }
    preg_match('/^version\s*=\s*"([^"]+)"/m', file_get_contents(__DIR__.'/../Cargo.toml'), $version);
    if (Phpolars\version() !== ($version[1] ?? null)) {
        throw new RuntimeException('unexpected extension version');
    }
    $df = new Phpolars\DataFrame(['g' => ['a', 'b', 'a'], 'x' => [1, 2, 3]]);
    $out = $df->groupBy('g', maintainOrder: true)->agg(col('x')->sum())->toDicts();
    if ($out !== [['g' => 'a', 'x' => 4], ['g' => 'b', 'x' => 2]]) {
        throw new RuntimeException('group_by/agg failed');
    }
    $threads = Phpolars\thread_pool_size();
    $expected = (int) (getenv('POLARS_MAX_THREADS') ?: ini_get('php_polars.max_threads'));
    if ($threads !== $expected) {
        throw new RuntimeException("thread pool has {$threads} threads, expected {$expected}");
    }
    if (PHP_SAPI !== 'cli') {
        header('Content-Type: application/json');
    }
    echo json_encode(['ok' => true, 'sapi' => PHP_SAPI, 'php' => PHP_VERSION, 'extension' => Phpolars\version(),
        'zts' => (bool) PHP_ZTS, 'threads' => $threads, 'result' => $out], JSON_THROW_ON_ERROR), "\n";
} catch (Throwable $error) {
    if (PHP_SAPI !== 'cli') {
        http_response_code(500);
        header('Content-Type: application/json');
    }
    echo json_encode(['ok' => false, 'error' => $error->getMessage()], JSON_THROW_ON_ERROR), "\n";
    exit(1);
}
