<?php
$orders = new DataFrame([
    'order' => [1, 2, 3],
    'items' => [['apple', 'banana'], ['cherry'], []],
    'ship' => [['city' => 'Baltimore', 'days' => 2], ['city' => 'Denver', 'days' => 5], null],
]);

echo $orders, "\n";
echo $orders->select(
    'order',
    col('items')->list()->len()->alias('n_items'),
    col('items')->list()->join(', ')->alias('items'),
    col('ship')->struct()->field('city'),
), "\n";
echo $orders->explode('items'), "\n";
print_r($orders->row(0, named: true));
