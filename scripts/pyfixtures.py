"""Generate tests/fixtures/py/expected.json from py-polars, the reference API.

Run with the py-polars release whose Rust core matches the crate in Cargo.toml
(rs-0.55.2 was cut 26 commits after py-1.43.2):

    uv run --with 'polars==1.43.2' python scripts/pyfixtures.py

tests/pyfixtures.php defines the same cases with the phpolars API and compares
its output with the file this writes. Keep the case names in sync.
"""

from __future__ import annotations

import datetime as dt
import json
import math
import warnings
from pathlib import Path

import polars as pl

warnings.filterwarnings("ignore", category=DeprecationWarning)

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "tests" / "fixtures" / "py" / "expected.json"

col, lit, when = pl.col, pl.lit, pl.when

# ---------------------------------------------------------------------------
# Inputs. tests/pyfixtures.php builds the same frames.

base = (
    pl.DataFrame(
        {
            "id": [1, 2, 3, 4, 5, 6],
            "g": ["a", "b", "a", "c", None, "b"],
            "x": [3, None, 1, 7, 2, 5],
            "y": [1.5, 2.25, None, -0.5, 10.0, 3.0],
            "k": [2, 1, 2, None, 3, 1],
            "b": [True, False, None, True, False, True],
            "s": ["apple pie", "Bänana", None, "cherry-42", "  date  ", "elder_7 berry"],
            "ds": ["2024-01-15", "2024-02-29", None, "2023-12-31", "2024-07-04", "2025-03-09"],
            "ts": [
                "2024-01-15 10:30:00",
                "2024-02-29 23:59:59",
                "2023-06-01 00:00:01",
                None,
                "2024-07-04 12:00:00",
                "2025-03-09 06:15:30",
            ],
            "l": [[1, 2, 3], [], None, [4, None], [5], [6, 7]],
        }
    )
    .with_columns(
        col("ds").str.to_date("%Y-%m-%d").alias("d"),
        col("ts").str.to_datetime("%Y-%m-%d %H:%M:%S").alias("t"),
    )
    .select("id", "g", "x", "y", "k", "b", "s", "d", "t", "l")
)

right = pl.DataFrame({"g": ["a", "c", "d", None], "z": [10, 30, 40, 99], "x": [100, 300, 400, 999]})

long = pl.DataFrame(
    {
        "id": [1, 1, 2, 2, 3, 1],
        "k": ["p", "q", "p", "q", "p", "p"],
        "v": [10, 20, 30, None, 50, 5],
    }
)

nums = pl.DataFrame({"n": ["1", "20", "-3", "x", None, "007"]})
rnd = pl.DataFrame({"f": [0.5, 1.5, 2.5, -0.5, -1.5, 1.005, 2.675, None]})

# ---------------------------------------------------------------------------
# Cases. Each returns a DataFrame, a Series, or a dict of scalars.

CASES: dict[str, callable] = {}


def case(name):
    def deco(f):
        CASES[name] = f
        return f

    return deco


def simple(name, f):
    CASES[name] = f


# select / with_columns: arithmetic and comparison
simple("select_arith", lambda: base.select(
    "id",
    (col("x") + 1).alias("add"),
    (col("x") - col("id")).alias("sub"),
    (col("x") * col("y")).alias("mul"),
    (col("x") / 2).alias("truediv"),
    (col("x") // 2).alias("floordiv"),
    (col("x") % 3).alias("mod"),
    (col("x") ** 2).alias("pow"),
    (-col("y")).alias("neg"),
))
simple("select_float_specials", lambda: base.select(
    (col("y") / 0).alias("div0"),
    (col("x") / 0).alias("idiv0"),
    (col("x") // 0).alias("ifloordiv0"),
    (col("y") * 0 / 0).alias("nan"),
))
simple("select_int_div_neg", lambda: pl.DataFrame({"a": [7, -7, 7, -7], "b": [2, 2, -2, -2]}).select(
    (col("a") // col("b")).alias("fd"), (col("a") % col("b")).alias("m"), (col("a") / col("b")).alias("td"),
))
simple("select_literals", lambda: base.head(2).select(
    lit(1).alias("i"), lit(2.5).alias("f"), lit("k").alias("s"), lit(True).alias("t"), lit(None).alias("n"),
))
simple("select_literal_broadcast", lambda: base.select("id", lit(0).alias("z"), (col("x") + lit(0.5)).alias("xf")))
simple("int32_plus_literal", lambda: base.select(
    (col("x").cast(pl.Int32) + 1).alias("a"),
    (col("x").cast(pl.Int32) + col("id")).alias("b"),
    (col("x").cast(pl.UInt8) * 2).alias("c"),
    (col("y").cast(pl.Float32) * 2).alias("d"),
))
simple("with_columns_compare", lambda: base.select("id", "x", "y", "g").with_columns(
    (col("x") > 2).alias("gt"),
    (col("x") >= 3).alias("ge"),
    (col("y") < 2).alias("lt"),
    (col("y") <= 2.25).alias("le"),
    (col("g") == "a").alias("eq"),
    (col("g") != "a").alias("ne"),
    col("x").eq_missing(None).alias("eqm"),
    col("g").ne_missing("a").alias("nem"),
    (col("x") > col("y")).alias("cross"),
))
simple("with_columns_overwrite", lambda: base.select("id", "x", "g").with_columns((col("x") * 10).alias("x"), col("g").alias("g2")))
simple("bool_logic", lambda: base.select(
    "b",
    (col("b") & (col("x") > 2)).alias("and"),
    (col("b") | (col("x") > 2)).alias("or"),
    (~col("b")).alias("not"),
    (col("b") ^ (col("x") > 2)).alias("xor"),
))
simple("horizontal_fns", lambda: base.select(
    pl.sum_horizontal("x", "y").alias("sum"),
    pl.mean_horizontal("x", "y").alias("mean"),
    pl.min_horizontal("x", "y").alias("min"),
    pl.max_horizontal("x", "k").alias("max"),
    pl.coalesce("x", "k", lit(-1)).alias("coal"),
    pl.concat_str(["g", "x"], separator="-").alias("cs"),
    pl.concat_str(["g", "x"], separator="-", ignore_nulls=True).alias("csi"),
))

# filter
simple("filter_gt", lambda: base.select("id", "x").filter(col("x") > 2))
simple("filter_multi", lambda: base.select("id", "g", "x").filter(col("x") > 1, col("g") == "b"))
simple("filter_or_null", lambda: base.select("id", "g", "y").filter(col("g").is_null() | (col("y") < 0)))
simple("filter_bool_col", lambda: base.select("id", "b").filter(col("b")))
simple("filter_date", lambda: base.select("id", "d").filter(col("d") > lit(dt.date(2024, 1, 31))))
simple("filter_datetime", lambda: base.select("id", "t").filter(col("t") >= dt.datetime(2024, 2, 1)))

# sort
simple("sort_single", lambda: base.select("id", "x").sort("x"))
simple("sort_desc_nulls_last", lambda: base.select("id", "y").sort("y", descending=True, nulls_last=True))
simple("sort_multi", lambda: base.select("id", "g", "x").sort(["g", "x"], descending=[False, True]))
simple("sort_multi_nulls_last", lambda: base.select("id", "g", "k").sort(["g", "k"], nulls_last=True))
simple("sort_multi_mixed_nulls_last", lambda: base.select("id", "g", "k").sort(["g", "k"], descending=[True, False], nulls_last=[True, False]))
simple("sort_expr", lambda: base.select("id", "s").sort(col("s").str.len_chars(), descending=True))
simple("sort_date_desc", lambda: base.select("id", "d").sort("d", descending=True))
simple("sort_string", lambda: base.select("id", "s").sort("s"))
simple("sort_bool", lambda: base.select("id", "b").sort("b", "id"))

# group_by
G = lambda *keys: base.group_by(*keys, maintain_order=True)
simple("gb_sum_mean", lambda: G("g").agg(col("x").sum().alias("sx"), col("y").sum().alias("sy"), col("x").mean().alias("mx"), col("y").mean().alias("my")))
simple("gb_median_min_max", lambda: G("g").agg(
    col("x").median().alias("med"), col("x").min().alias("min"), col("x").max().alias("max"),
    col("s").min().alias("smin"), col("d").max().alias("dmax"), col("t").min().alias("tmin"),
))
simple("gb_std_var", lambda: G("g").agg(col("y").std().alias("std"), col("y").var().alias("var"), col("x").std(ddof=0).alias("std0")))
simple("gb_count_len", lambda: G("g").agg(col("x").count().alias("count"), pl.len().alias("len"), col("x").null_count().alias("nulls")))
simple("gb_n_unique_first_last", lambda: G("b").agg(col("g").n_unique().alias("nu"), col("x").first().alias("first"), col("x").last().alias("last")))
simple("gb_quantile", lambda: G("b").agg(
    col("id").quantile(0.4, "nearest").alias("nearest"),
    col("id").quantile(0.4, "linear").alias("linear"),
    col("id").quantile(0.4, "lower").alias("lower"),
    col("id").quantile(0.4, "higher").alias("higher"),
    col("id").quantile(0.4, "midpoint").alias("midpoint"),
))
simple("gb_multi_key", lambda: G("g", "b").agg(col("id").sum().alias("ids")))
simple("gb_expr_key", lambda: G((col("x") > 2).alias("big")).agg(col("id").alias("ids"), col("y").max()))
simple("gb_implicit_list", lambda: G("g").agg("x", col("s").sort().first().alias("s1")))
simple("gb_filter_sortby", lambda: G("g").agg(
    col("x").filter(col("y") > 1).sum().alias("fs"),
    col("id").sort_by("y", descending=True).first().alias("top"),
))
simple("gb_len_method", lambda: G("g").len())
simple("gb_sum_method", lambda: base.select("g", "x", "y").group_by("g", maintain_order=True).sum())
simple("gb_mean_method", lambda: base.select("b", "x", "y").group_by("b", maintain_order=True).mean())
simple("gb_head", lambda: base.select("g", "id").group_by("g", maintain_order=True).head(1))

# joins
L = base.select("id", "g", "x")
simple("join_inner", lambda: L.join(right, on="g", suffix="_r").sort("id"))
simple("join_left", lambda: L.join(right.select("g", "z"), on="g", how="left", maintain_order="left"))
simple("join_full", lambda: L.join(right.select("g", "z"), on="g", how="full").sort("id", "z"))
simple("join_full_coalesce", lambda: L.join(right.select("g", "z"), on="g", how="full", coalesce=True).sort("id", "z"))
simple("join_semi", lambda: L.join(right, on="g", how="semi", maintain_order="left"))
simple("join_anti", lambda: L.join(right, on="g", how="anti", maintain_order="left"))
simple("join_nulls_equal", lambda: L.join(right, on="g", how="left", nulls_equal=True, maintain_order="left"))
simple("join_cross", lambda: L.head(2).join(right.select("z").head(3), how="cross", maintain_order="left_right"))
simple("join_suffix", lambda: L.join(right, on="g", suffix="_r", maintain_order="left"))
simple("join_left_on_right_on", lambda: L.join(right.rename({"g": "key"}), left_on="g", right_on="key", how="left", maintain_order="left"))
simple("join_multi_key", lambda: base.select("id", "g", "b").join(
    pl.DataFrame({"g": ["a", "b", "c"], "b": [True, True, True], "w": [1.5, 2.5, 3.5]}), on=["g", "b"], how="left", maintain_order="left",
))

# unique
simple("unique_subset_first", lambda: base.select("id", "g").unique(subset="g", keep="first", maintain_order=True))
simple("unique_keep_last", lambda: base.select("id", "g").unique(subset="g", keep="last", maintain_order=True))
simple("unique_keep_none", lambda: base.select("id", "g").unique(subset="g", keep="none", maintain_order=True))
simple("unique_rows", lambda: long.select("id", "k").unique(maintain_order=True))

# nulls
simple("fill_null_value_frame", lambda: base.select("g", "x", "y", "b").fill_null(0))
simple("fill_null_string", lambda: base.select(col("g").fill_null("?"), col("s").fill_null(lit("none"))))
simple("fill_null_strategies", lambda: base.select(
    col("x").fill_null(strategy="forward").alias("fwd"),
    col("y").fill_null(strategy="backward").alias("bwd"),
    col("x").fill_null(strategy="min").alias("min"),
    col("x").fill_null(strategy="max").alias("max"),
    col("x").fill_null(strategy="mean").alias("mean"),
    col("y").fill_null(strategy="mean").alias("ymean"),
    col("x").fill_null(strategy="zero").alias("zero"),
    col("x").fill_null(strategy="one").alias("one"),
))
simple("fill_null_limit", lambda: pl.DataFrame({"a": [1, None, None, None, 5, None]}).select(
    col("a").fill_null(strategy="forward", limit=1).alias("f1"),
    col("a").fill_null(strategy="backward", limit=2).alias("b2"),
))
simple("fill_null_frame_forward", lambda: base.select("g", "x", "d").fill_null(strategy="forward"))
simple("drop_nulls_all", lambda: base.select("id", "g", "x", "y").drop_nulls())
simple("drop_nulls_subset", lambda: base.select("id", "g", "x", "y").drop_nulls(["x", "g"]))
simple("null_count", lambda: base.null_count())
simple("interpolate", lambda: pl.DataFrame({"a": [1.0, None, None, 4.0, None], "i": [1, None, 3, None, 5]}).select(
    col("a").interpolate(), col("i").interpolate(),
))

# reshape
simple("unpivot", lambda: base.select("id", "x", "k").unpivot(index="id"))
simple("unpivot_mixed_names", lambda: base.select("id", "x", "y").unpivot(on=["x", "y"], index="id", variable_name="var", value_name="val"))
simple("pivot_first", lambda: long.pivot("k", index="id", values="v", aggregate_function="first"))
simple("pivot_sum", lambda: long.pivot("k", index="id", values="v", aggregate_function="sum"))
simple("pivot_mean", lambda: long.pivot("k", index="id", values="v", aggregate_function="mean"))
simple("pivot_len", lambda: long.pivot("k", index="id", values="v", aggregate_function="len"))
simple("explode_single", lambda: base.select("id", "l").explode("l"))
simple("explode_two", lambda: pl.DataFrame({"a": [[1, 2], [3]], "b": [["x", "y"], ["z"]], "c": [0, 1]}).explode("a", "b"))
simple("transpose", lambda: base.select("x", "k").head(3).transpose(include_header=True))
simple("with_row_index", lambda: base.select("g").with_row_index("i", offset=10))

# concat
A = pl.DataFrame({"a": [1, 2], "b": ["x", "y"]})
B = pl.DataFrame({"a": [3], "b": ["z"]})
C = pl.DataFrame({"b": ["w"], "c": [1.5]})
C2 = pl.DataFrame({"b2": ["v", "w"], "c": [1.5, None]})
simple("concat_vertical", lambda: pl.concat([A, B]))
simple("concat_horizontal", lambda: pl.concat([A, C2], how="horizontal"))
simple("concat_diagonal", lambda: pl.concat([A, C], how="diagonal"))
simple("concat_vertical_relaxed", lambda: pl.concat([A, pl.DataFrame({"a": [1.5], "b": ["q"]})], how="vertical_relaxed"))
simple("concat_diagonal_relaxed", lambda: pl.concat([A, pl.DataFrame({"a": [0.5], "c": [True]})], how="diagonal_relaxed"))

# describe
simple("describe_numeric", lambda: base.select("x", "y").describe())
simple("describe_mixed", lambda: base.select("x", "g", "b", "d").describe())
simple("describe_percentiles", lambda: base.select("y").describe(percentiles=[0.1, 0.9]))

# frame aggregates
simple("frame_sum", lambda: base.select("g", "x", "y", "b").sum())
simple("frame_mean", lambda: base.select("x", "y", "b").mean())
simple("frame_min_max", lambda: pl.concat([base.select("g", "x", "y", "d").min(), base.select("g", "x", "y", "d").max()]))
simple("frame_std_var", lambda: pl.concat([base.select("x", "y").std(), base.select("x", "y").var(ddof=0)]))
simple("frame_median", lambda: base.select("x", "y").median())
simple("frame_n_unique", lambda: base.select(col("g", "x", "b").n_unique()))

# windows
simple("over_sum", lambda: base.select("id", col("x").sum().over("g").alias("sx")))
simple("over_multi", lambda: base.select("id", col("y").mean().over("g", "b").alias("m"), pl.len().over("g").alias("n")))
simple("over_order_by", lambda: base.select("id", col("x").cum_sum().over("g", order_by="id").alias("cs")))
simple("over_order_by_desc", lambda: base.select("id", col("id").first().over("g", order_by="y", descending=True, nulls_last=True).alias("f")))
simple("over_rank", lambda: base.select("id", col("y").rank("dense").over("b").alias("r")))
simple("cum_fns", lambda: base.select(
    col("x").cum_sum().alias("sum"),
    col("x").cum_prod().alias("prod"),
    col("y").cum_min().alias("min"),
    col("y").cum_max().alias("max"),
    col("x").cum_count().alias("count"),
    col("x").cum_sum(reverse=True).alias("rsum"),
))
simple("shift", lambda: base.select(
    col("x").shift().alias("s1"),
    col("x").shift(-2).alias("sm2"),
    col("g").shift(1, fill_value="-").alias("sf"),
    col("d").shift(1).alias("sd"),
))
simple("diff", lambda: base.select(col("x").diff().alias("d1"), col("y").diff(2).alias("d2"), col("t").diff().alias("dt")))
simple("pct_change", lambda: base.select(col("y").pct_change().alias("p")))
simple("rank_methods", lambda: base.select(
    "k",
    col("k").rank("average").alias("avg"),
    col("k").rank("min").alias("min"),
    col("k").rank("max").alias("max"),
    col("k").rank("dense").alias("dense"),
    col("k").rank("ordinal").alias("ord"),
    col("k").rank("dense", descending=True).alias("dd"),
))
simple("rolling", lambda: base.select(
    col("x").rolling_mean(2).alias("m2"),
    col("x").rolling_mean(3, min_samples=1).alias("m3"),
    col("y").rolling_mean(3, center=True).alias("mc"),
    col("id").rolling_sum(3).alias("s3"),
    col("id").rolling_min(2).alias("min2"),
    col("id").rolling_max(2).alias("max2"),
    col("id").rolling_std(3).alias("std3"),
))

# when / then / otherwise
simple("when_basic", lambda: base.select("id", when(col("x") > 2).then(lit("big")).otherwise(lit("small")).alias("w")))
simple("when_chain", lambda: base.select(
    "id",
    when(col("x") < 2).then(lit(1)).when(col("x") < 5).then(lit(2)).otherwise(lit(3)).alias("w"),
))
simple("when_no_otherwise", lambda: base.select("id", when(col("g") == "a").then(col("y")).alias("w")))
simple("when_multi_cond", lambda: base.select("id", when(col("x") > 1, col("b")).then(col("x")).otherwise(col("k")).alias("w")))
simple("when_supertype", lambda: base.select("id", when(col("b")).then(col("x")).otherwise(col("y")).alias("w")))

# str namespace
S = lambda *e: base.select(*e)
s = col("s").str
simple("str_contains", lambda: S(
    s.contains("a").alias("re"),
    s.contains("-", literal=True).alias("lit"),
    s.contains(r"\d+").alias("digits"),
    s.contains("(?i)BANANA").alias("ci"),
    s.contains(".", literal=True).alias("dot"),
))
simple("str_replace", lambda: S(
    s.replace("a", "A").alias("r1"),
    s.replace_all("a", "A").alias("ra"),
    s.replace(r"(\w+)-(\d+)", "$2-$1").alias("grp"),
    s.replace_all(".", "!", literal=True).alias("lit"),
    s.replace_all(r"\s+", "_").alias("ws"),
))
simple("str_slice", lambda: S(s.slice(1, 3).alias("a"), s.slice(-3).alias("b"), s.head(2).alias("h"), s.tail(2).alias("t")))
simple("str_split", lambda: S(s.split(" ").alias("sp"), s.split("-", inclusive=True).alias("inc")))
simple("str_strip", lambda: S(s.strip_chars().alias("a"), s.strip_chars("ae ").alias("b"), s.strip_chars_start().alias("c"), s.strip_chars_end("y").alias("d")))
simple("str_case", lambda: S(s.to_uppercase().alias("u"), s.to_lowercase().alias("l")))
simple("str_len", lambda: S(s.len_chars().alias("chars"), s.len_bytes().alias("bytes")))
simple("str_pad", lambda: S(s.pad_start(10, "*").alias("ps"), s.pad_end(10).alias("pe"), col("x").cast(pl.String).str.zfill(3).alias("z")))
simple("str_extract", lambda: S(
    s.extract(r"(\d+)", 1).alias("num"),
    s.extract(r"^(\w)(\w)", 2).alias("second"),
    s.extract_all(r"[aeiou]").alias("vowels"),
))
simple("str_count_matches", lambda: S(s.count_matches("e").alias("e"), s.count_matches(".", literal=True).alias("dots"), s.count_matches(r"\w+").alias("words")))
simple("str_starts_ends", lambda: S(s.starts_with("a").alias("sw"), s.ends_with("y").alias("ew")))
simple("str_to_integer", lambda: nums.select(col("n").str.to_integer(strict=False).alias("i")))
simple("str_to_integer_base", lambda: pl.DataFrame({"h": ["ff", "10", "7f"]}).select(col("h").str.to_integer(base=16)))
simple("str_find_reverse", lambda: S(s.find("e").alias("f"), s.reverse().alias("r")))
simple("str_join_agg", lambda: S(s.join("|").alias("j"), s.join("|", ignore_nulls=False).alias("jn")))
simple("str_to_date_datetime", lambda: pl.DataFrame({"a": ["2024-03-05", "2021-12-31"], "b": ["05/03/2024 13:14", "31/12/2021 00:00"]}).select(
    col("a").str.to_date(), col("b").str.to_datetime("%d/%m/%Y %H:%M"),
))

# dt namespace
t, d = col("t").dt, col("d").dt
simple("dt_parts", lambda: S(
    t.year().alias("y"), t.month().alias("m"), t.day().alias("d"), t.weekday().alias("wd"),
    t.hour().alias("h"), t.minute().alias("mi"), t.second().alias("s"),
    t.ordinal_day().alias("od"), t.quarter().alias("q"), t.week().alias("w"),
))
simple("date_parts", lambda: S(d.year().alias("y"), d.month().alias("m"), d.day().alias("dd"), d.weekday().alias("wd"), d.is_leap_year().alias("leap")))
simple("dt_strftime", lambda: S(t.strftime("%Y/%m/%d %H:%M").alias("a"), d.strftime("%d %b %Y").alias("b"), t.to_string("%H-%M-%S").alias("c")))
simple("dt_truncate", lambda: S(t.truncate("1mo").alias("mo"), t.truncate("1w").alias("w"), t.truncate("1d").alias("d"), t.truncate("1h").alias("h"), d.truncate("1mo").alias("dmo")))
simple("dt_round", lambda: S(t.round("1h").alias("h")))
simple("dt_epoch", lambda: S(t.epoch("s").alias("s"), t.epoch("ms").alias("ms"), t.epoch().alias("us"), d.epoch("d").alias("days"), t.timestamp("ms").alias("ts")))
simple("dt_date_time", lambda: S(t.date().alias("date"), t.time().alias("time"), d.month_start().alias("ms"), d.month_end().alias("me")))
simple("dt_offset_by", lambda: S(d.offset_by("1mo").alias("a"), t.offset_by("-2d").alias("b"), d.offset_by("1y").alias("c")))
simple("dt_duration", lambda: S((col("t") - col("t").min()).alias("dur"), (col("t") - col("t").min()).dt.total_hours().alias("hrs"), (col("d") - col("d").min()).dt.total_days().alias("days")))
simple("dt_cast_time_unit", lambda: S(t.cast_time_unit("ms").alias("ms")))
simple("dt_time_zone", lambda: S(t.replace_time_zone("UTC").dt.convert_time_zone("Asia/Tokyo").dt.hour().alias("h"), t.replace_time_zone("UTC").alias("utc")))

# list namespace
li = col("l").list
simple("list_aggs", lambda: S(li.len().alias("len"), li.sum().alias("sum"), li.mean().alias("mean"), li.min().alias("min"), li.max().alias("max")))
simple("list_get", lambda: S(li.get(0, null_on_oob=True).alias("g0"), li.get(-1, null_on_oob=True).alias("gl"), li.first().alias("f"), li.last().alias("la")))
simple("list_contains", lambda: S(li.contains(2).alias("c2"), li.contains(None).alias("cn")))
simple("list_join", lambda: S(col("s").str.split(" ").list.join("+").alias("j")))
simple("list_eval", lambda: S(li.eval(pl.element() * 10).alias("x10"), li.eval(pl.element().rank()).alias("rank")))
simple("list_sort_unique", lambda: S(li.sort(descending=True).alias("sd"), li.reverse().alias("rv"), li.slice(1, 2).alias("sl"), li.head(1).alias("h")))
simple("list_unique", lambda: pl.DataFrame({"a": [[3, 1, 3, 2], [None, None]]}).select(col("a").list.unique(maintain_order=True), col("a").list.n_unique().alias("nu")))
simple("list_concat_list", lambda: S(pl.concat_list("x", "k").alias("cl")))

# cast
simple("cast_numeric", lambda: S(
    col("x").cast(pl.Float64).alias("xf"),
    col("y").cast(pl.Int64).alias("yi"),
    col("y").cast(pl.Int32).alias("y32"),
    col("b").cast(pl.Int8).alias("b8"),
    col("x").cast(pl.Boolean).alias("xb"),
))
simple("cast_to_string", lambda: S(
    col("x").cast(pl.String).alias("xs"),
    col("y").cast(pl.String).alias("ys"),
    col("b").cast(pl.String).alias("bs"),
    col("d").cast(pl.String).alias("ds"),
    col("t").cast(pl.String).alias("ts"),
))
simple("cast_temporal", lambda: S(col("d").cast(pl.Datetime).alias("dt"), col("t").cast(pl.Date).alias("date"), col("d").cast(pl.Int32).alias("di")))
simple("cast_strict_fails", lambda: S(col("s").cast(pl.Int64)))
simple("cast_non_strict", lambda: S(col("s").cast(pl.Int64, strict=False).alias("si"), (col("x") * 50).cast(pl.UInt8, strict=False).alias("u8")))
simple("cast_overflow_strict_fails", lambda: S((col("x") * 50).cast(pl.UInt8)))
simple("cast_string_to_float", lambda: pl.DataFrame({"a": ["1.5", "-2", "1e3", "x", None]}).select(col("a").cast(pl.Float64, strict=False)))
simple("cast_frame_map", lambda: base.select("x", "y", "k").cast({"x": pl.Float32, "k": pl.String}))

# is_in / is_between / clip / round
simple("is_in", lambda: S(
    col("x").is_in([1, 2, 3]).alias("x"),
    col("g").is_in(["a", "c"]).alias("g"),
    col("g").is_in(["a", None], nulls_equal=True).alias("gn"),
    col("y").is_in([1.5, 3.0]).alias("y"),
))
simple("is_between", lambda: S(
    col("x").is_between(2, 5).alias("both"),
    col("x").is_between(2, 5, closed="left").alias("left"),
    col("x").is_between(2, 5, closed="right").alias("right"),
    col("x").is_between(2, 5, closed="none").alias("none"),
    col("y").is_between(0, 2.25).alias("f"),
))
simple("is_between_dates", lambda: S(col("t").is_between(dt.datetime(2024, 1, 1), dt.datetime(2024, 12, 31)).alias("t")))
simple("clip", lambda: S(
    col("x").clip(2, 5).alias("both"),
    col("y").clip(lower_bound=0).alias("lo"),
    col("y").clip(upper_bound=2).alias("hi"),
    col("x").clip(1.5, 4.5).alias("fb"),
))
simple("round", lambda: rnd.select(
    col("f").round(0).alias("r0"),
    col("f").round(1).alias("r1"),
    col("f").round(2).alias("r2"),
    col("f").round(0, mode="half_away_from_zero").alias("away"),
    col("f").floor().alias("floor"),
    col("f").ceil().alias("ceil"),
))
simple("math", lambda: S(col("y").abs().alias("abs"), col("x").sqrt().alias("sqrt"), col("x").log().alias("ln"), col("x").log(2).alias("log2"), col("x").exp().alias("exp"), col("y").sign().alias("sign")))

# value_counts
simple("value_counts_expr", lambda: base.select(col("g").value_counts(sort=True)).unnest("g").sort("count", "g", descending=[True, False], nulls_last=True))
simple("value_counts_series", lambda: base.get_column("k").value_counts(sort=True).sort("count", "k", descending=[True, False], nulls_last=True))
simple("value_counts_normalize", lambda: base.get_column("b").value_counts(normalize=True, name="share").sort("b"))

# Series
xs, ys, gs, bs = (base.get_column(c) for c in ("x", "y", "g", "b"))
simple("series_int_aggs", lambda: {
    "sum": xs.sum(), "mean": xs.mean(), "median": xs.median(), "min": xs.min(), "max": xs.max(),
    "std": xs.std(), "var": xs.var(), "product": xs.product(), "count": xs.count(), "len": xs.len(),
    "null_count": xs.null_count(), "n_unique": xs.n_unique(), "first": xs.first(), "last": xs.last(),
    "arg_min": xs.arg_min(), "arg_max": xs.arg_max(),
})
simple("series_float_aggs", lambda: {
    "sum": ys.sum(), "mean": ys.mean(), "median": ys.median(), "min": ys.min(), "max": ys.max(),
    "std": ys.std(), "std0": ys.std(ddof=0), "var": ys.var(), "product": ys.product(),
    "q25": ys.quantile(0.25), "q25_linear": ys.quantile(0.25, "linear"), "q75_nearest": ys.quantile(0.75, "nearest"),
    "first": ys.first(), "last": ys.last(),
})
simple("series_quantile_int_arg", lambda: {"q0": ys.quantile(0), "q1": ys.quantile(1)})
simple("series_str_aggs", lambda: {"min": gs.min(), "max": gs.max(), "n_unique": gs.n_unique(), "count": gs.count(), "first": gs.first(), "last": gs.last()})
simple("series_bool_aggs", lambda: {"any": bs.any(), "all": bs.all(), "all_nulls": bs.all(ignore_nulls=False), "sum": bs.sum(), "mean": bs.mean()})
simple("series_ops", lambda: pl.DataFrame([
    (xs * 2).alias("mul"), xs.cum_sum().alias("cs"), xs.diff().alias("diff"), xs.shift(1).alias("shift"),
    xs.rank().alias("rank"), xs.clip(2, 5).alias("clip"), xs.is_in([1, 7]).alias("isin"), ys.round(1).alias("round"),
]))
simple("series_sort", lambda: xs.sort(descending=True, nulls_last=True))
simple("series_unique", lambda: gs.unique(maintain_order=True))
simple("series_empty_aggs", lambda: (lambda e: {"sum": e.sum(), "mean": e.mean(), "min": e.min(), "max": e.max(), "std": e.std()})(pl.Series("e", [], pl.Int64)))

# misc frame ops
simple("head_tail_slice", lambda: pl.concat([base.select("id").head(2), base.select("id").tail(1), base.select("id").slice(-3, 2)]))
simple("rename_drop_reverse", lambda: base.select("id", "g", "x").rename({"g": "group"}).drop("x").reverse())
simple("struct_pack_field", lambda: base.select(pl.struct("g", "x").alias("st")).with_columns(col("st").struct.field("x").alias("sx")))
simple("unique_counts_mode", lambda: base.select(col("k").drop_nulls().mode().sort().alias("mode")))
simple("top_bottom_k", lambda: base.select(col("y").top_k(2).alias("top"), col("y").bottom_k(2).alias("bottom")))


# ---------------------------------------------------------------------------
# Serialisation


def dtype_name(t: pl.DataType) -> str:
    """The phpolars dtype string: 'Datetime(us)', 'List(Int64)', 'Struct(a: Int64)'."""
    if isinstance(t, pl.Datetime):
        return f"Datetime({t.time_unit}, {t.time_zone})" if t.time_zone else f"Datetime({t.time_unit})"
    if isinstance(t, pl.Duration):
        return f"Duration({t.time_unit})"
    if isinstance(t, pl.List):
        return f"List({dtype_name(t.inner)})"
    if isinstance(t, pl.Array):
        return f"Array({dtype_name(t.inner)}, {t.size})"
    if isinstance(t, pl.Struct):
        return "Struct(" + ", ".join(f"{f.name}: {dtype_name(f.dtype)}" for f in t.fields) + ")"
    return str(t)


def scalar_tag(v) -> str:
    """PHP gettype()-style tag for a scalar result."""
    if v is None:
        return "null"
    if isinstance(v, bool):
        return "bool"
    if isinstance(v, int):
        return "int"
    if isinstance(v, float):
        return "float"
    if isinstance(v, str):
        return "string"
    return type(v).__name__


def encode(v):
    if isinstance(v, float) and not math.isfinite(v):
        return {"$float": "NaN" if math.isnan(v) else ("INF" if v > 0 else "-INF")}
    if isinstance(v, dt.datetime):
        return v.strftime("%Y-%m-%dT%H:%M:%S.%f") + (v.strftime("%z") if v.tzinfo else "")
    if isinstance(v, dt.date):
        return v.isoformat()
    if isinstance(v, dt.time):
        return v.strftime("%H:%M:%S.%f")
    if isinstance(v, dt.timedelta):
        # phpolars returns Duration as an int in the column's unit; every duration here is in us.
        return (v.days * 86_400 + v.seconds) * 1_000_000 + v.microseconds
    if isinstance(v, list):
        return [encode(x) for x in v]
    if isinstance(v, dict):
        return {k: encode(x) for k, x in v.items()}
    return v


def run(f):
    try:
        out = f()
    except Exception as e:  # noqa: BLE001 - an expected error is a fixture too
        return {"error": type(e).__name__, "message": str(e).splitlines()[0]}
    if isinstance(out, pl.Series):
        out = out.to_frame()
    if isinstance(out, dict):
        return {
            "columns": list(out),
            "dtypes": [scalar_tag(v) for v in out.values()],
            "rows": [[encode(v) for v in out.values()]],
            "scalar": True,
        }
    return {
        "columns": out.columns,
        "dtypes": [dtype_name(t) for t in out.dtypes],
        "rows": [[encode(v) for v in row] for row in out.rows()],
    }


def main():
    cases = {name: run(f) for name, f in CASES.items()}
    OUT.parent.mkdir(parents=True, exist_ok=True)
    doc = {"polars_version": pl.__version__, "cases": cases}
    OUT.write_text(json.dumps(doc, indent=1, ensure_ascii=False, allow_nan=False) + "\n")
    errors = [n for n, c in cases.items() if "error" in c]
    print(f"wrote {len(cases)} cases ({len(errors)} expected errors: {', '.join(errors)}) with polars {pl.__version__} to {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
