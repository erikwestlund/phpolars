"""Generate tests/fixtures/py/expected.json from py-polars, the reference API.

Run with the py-polars release whose Rust core matches the crate in Cargo.toml
(rs-0.55.2 was cut 26 commits after py-1.43.2):

    uv run --with 'polars==1.43.2' --with numpy python scripts/pyfixtures.py

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
import polars.selectors as cs

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

# A sorted time series and quotes for asof joins, windows, and rolling.
tsf = pl.DataFrame(
    {
        "t": [
            dt.datetime(2024, 1, 1, 0), dt.datetime(2024, 1, 1, 1), dt.datetime(2024, 1, 1, 3),
            dt.datetime(2024, 1, 2, 0), dt.datetime(2024, 1, 2, 5), dt.datetime(2024, 1, 4, 0),
        ],
        "g": ["a", "b", "a", "a", "b", "a"],
        "v": [1, 2, 3, None, 5, 6],
        "f": [1.0, float("nan"), 2.5, 4.0, None, 8.0],
    }
)
quotes = pl.DataFrame(
    {
        "t": [dt.datetime(2024, 1, 1, 0, 30), dt.datetime(2024, 1, 1, 2), dt.datetime(2024, 1, 2, 4)],
        "g": ["a", "a", "b"],
        "px": [10.0, 11.0, 12.0],
    }
)
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

# asof joins, time windows, and rolling
simple("join_asof_backward", lambda: tsf.join_asof(quotes, on="t"))
simple("join_asof_forward_by", lambda: tsf.join_asof(quotes, on="t", by="g", strategy="forward", check_sortedness=False))
simple("join_asof_nearest_tolerance", lambda: tsf.join_asof(quotes, on="t", strategy="nearest", tolerance="2h"))
simple("join_asof_int_tolerance", lambda: pl.DataFrame({"k": [1, 5, 10]}).join_asof(
    pl.DataFrame({"k": [2, 4, 9], "r": ["a", "b", "c"]}), on="k", tolerance=1, allow_exact_matches=False))
simple("group_by_dynamic_day", lambda: tsf.group_by_dynamic("t", every="1d").agg(col("v").sum(), pl.len()))
simple("group_by_dynamic_by_bounds", lambda: tsf.group_by_dynamic(
    "t", every="12h", period="1d", closed="both", group_by="g", include_boundaries=True).agg(col("v").max()))
simple("group_by_dynamic_label_offset", lambda: tsf.group_by_dynamic(
    "t", every="1d", offset="6h", label="right", start_by="datapoint").agg(col("v").first()))
simple("rolling_frame", lambda: tsf.rolling("t", period="1d").agg(col("v").sum().alias("s"), col("v").count().alias("n")))
simple("rolling_frame_by", lambda: tsf.rolling("t", period="2h", offset="-1h", closed="both", group_by="g").agg(col("v").sum()))
simple("rolling_by_exprs", lambda: tsf.select(
    "t",
    col("v").rolling_mean_by("t", "1d").alias("mean"),
    col("v").fill_null(0).rolling_sum_by("t", "3h", closed="both").alias("sum"),
    col("v").rolling_max_by("t", "2d", min_samples=2).alias("max"),
    col("v").rolling_min_by("t", "1d", closed="left").alias("min"),
    col("v").rolling_median_by("t", "1d").alias("median"),
    col("v").rolling_std_by("t", "1d").alias("std"),
    col("v").rolling_var_by("t", "1d", ddof=0).alias("var"),
    col("v").rolling_quantile_by("t", "1d", quantile=0.5).alias("q"),
))
simple("rolling_quantile_skew", lambda: base.select(
    col("y").rolling_quantile(0.5, window_size=2).alias("q"),
    col("x").rolling_quantile(0.25, "linear", window_size=3, min_samples=1).alias("ql"),
    col("id").rolling_skew(3).alias("skew"),
    col("id").cast(pl.Float64).pow(2).rolling_kurtosis(4).alias("kurt"),
))
simple("ewm", lambda: base.select(
    col("y").ewm_mean(alpha=0.5).alias("mean"),
    col("y").ewm_mean(span=3, adjust=False).alias("mean_span"),
    col("y").ewm_std(com=1).alias("std"),
    col("x").ewm_var(half_life=2, ignore_nulls=True, bias=True).alias("var"),
))
simple("ewm_mean_by", lambda: tsf.select(col("v").ewm_mean_by("t", half_life="1d")))
simple("ewm_no_alpha", lambda: base.select(col("y").ewm_mean()))

# binning, runs, and searching
simple("cut_qcut", lambda: base.select(
    "y",
    col("y").cut([0, 2]).alias("cut"),
    col("y").cut([0, 2], labels=["lo", "mid", "hi"], left_closed=True).alias("cutl"),
    col("y").qcut([0.5]).alias("q"),
    col("y").qcut(2, labels=["a", "b"]).alias("q2"),
).with_columns(pl.all().exclude("y").cast(pl.String)))
simple("cut_include_breaks", lambda: base.select(col("y").cut([0, 2], include_breaks=True).alias("c")).unnest("c").with_columns(col("category").cast(pl.String)))
simple("rle", lambda: pl.DataFrame({"a": [1, 1, 2, 2, 2, None, None, 1]}).select(col("a").rle().alias("r")).unnest("r"))
simple("rle_id", lambda: pl.DataFrame({"a": [1, 1, 2, 2, 2, None, None, 1]}).select(col("a").rle_id()))
simple("search_sorted", lambda: pl.DataFrame({"a": [1, 2, 2, 4, 7]}).select(
    col("a").search_sorted(2).alias("any"), col("a").search_sorted(2, "left").alias("left"),
    col("a").search_sorted(2, "right").alias("right"), col("a").search_sorted(9).alias("end"),
))
simple("search_sorted_many", lambda: pl.DataFrame({"a": [1, 2, 2, 4, 7]}).select(col("a").search_sorted([0, 3, 7], "left")))
simple("peaks_entropy", lambda: pl.DataFrame({"a": [1, 3, 2, 2, 5, 4]}).select(
    col("a").peak_max().alias("max"), col("a").peak_min().alias("min"),
    col("a").entropy().alias("e"), col("a").entropy(base=2, normalize=False).alias("e2"),
))
simple("hist_bins", lambda: pl.DataFrame({"a": [1.0, 2.5, 3.0, 4.5, 5.0, 2.0]}).select(col("a").hist(bins=[0, 2, 4, 6])))
simple("hist_count_breaks", lambda: pl.DataFrame({"a": [1.0, 2.5, 3.0, 4.5, 5.0, 2.0]}).select(
    col("a").hist(bin_count=2, include_breakpoint=True, include_category=True).alias("h")).unnest("h").with_columns(col("category").cast(pl.String)))
simple("interpolate_by", lambda: pl.DataFrame({"a": [1.0, None, 3.0, None, None, 6.0, None], "b": [1, 2, 5, 6, 7, 10, 11]}).select(col("a").interpolate_by("b")))
simple("is_close_index_of", lambda: pl.DataFrame({"a": [1.0, 1.0000000001, 2.0, float("nan")], "k": [3, 1, 3, 2]}).select(
    col("a").is_close(1.0).alias("close"),
    col("a").is_close(1.5, abs_tol=0.6).alias("abs"),
    col("a").is_close(float("nan"), nans_equal=True).alias("nan"),
    col("k").index_of(3).alias("idx"),
    col("k").index_of(9).alias("missing"),
))
simple("arg_unique_bounds", lambda: pl.DataFrame({"k": [3, 1, 3, 2, 1]}).select(
    col("k").arg_unique().alias("au").implode(), col("k").upper_bound().alias("ub"), col("k").cast(pl.Int8).lower_bound().alias("lb"),
))
simple("trig_extra", lambda: rnd.select(
    col("f").degrees().alias("deg"), col("f").radians().alias("rad"), col("f").sinh().alias("sinh"),
    col("f").cosh().alias("cosh"), col("f").tanh().alias("tanh"), col("f").arcsinh().alias("asinh"),
    col("f").abs().add(1).arccosh().alias("acosh"), (col("f") / 10).arctanh().alias("atanh"), col("f").cot().alias("cot"),
))

# frame verbs
simple("frame_shift", lambda: base.select("id", "x").shift(2))
simple("frame_shift_fill", lambda: base.select("id", "x").shift(-1, fill_value=0))
simple("frame_interpolate", lambda: base.select("x", "y").interpolate())
simple("frame_quantile_product", lambda: pl.concat([
    base.select("x", "y", "k").quantile(0.5, "linear"),
    base.select("x", "y", "k").cast(pl.Float64).product(),
]))
simple("frame_top_bottom_k", lambda: pl.concat([
    base.select("id", "y").top_k(2, by="y"),
    base.select("id", "k").bottom_k(3, by=["k", "id"]),
    base.select("id", "x").top_k(2, by="x", reverse=True),
], how="diagonal"))
simple("frame_gather_every", lambda: base.select("id", "g").gather_every(2, offset=1))
simple("frame_drop_nans", lambda: tsf.drop_nans())
simple("frame_drop_nans_subset", lambda: tsf.with_columns(col("f").alias("f2")).drop_nans(subset="f2"))
simple("frame_is_duplicated", lambda: long.select("id", "k").select(
    long.select("id", "k").is_duplicated().alias("dup"), long.select("id", "k").is_unique().alias("uniq")))
simple("frame_upsample", lambda: pl.DataFrame({
    "t": [dt.date(2024, 1, 1), dt.date(2024, 1, 4), dt.date(2024, 1, 2), dt.date(2024, 1, 3)],
    "g": ["a", "a", "b", "b"], "v": [1, 4, 2, 3],
}).sort("t").upsample("t", every="1d", group_by="g", maintain_order=True))
simple("frame_corr", lambda: base.select("id", "x", "y").drop_nulls().corr())
simple("frame_insert_to_struct", lambda: base.select("id", "g").head(3).insert_column(1, pl.Series("n", [7, 8, 9])).to_struct("st").to_frame().unnest("st"))
simple("frame_approx_n_unique", lambda: base.select("g", "k").approx_n_unique())

# functions
simple("fn_date_range", lambda: pl.select(pl.date_range(dt.date(2024, 1, 1), dt.date(2024, 1, 10), "3d").alias("d")))
simple("fn_date_range_closed", lambda: pl.select(pl.date_range(dt.date(2024, 1, 30), dt.date(2024, 4, 30), "1mo", closed="left").alias("d")))
simple("fn_datetime_range", lambda: pl.select(pl.datetime_range(dt.datetime(2024, 1, 1), dt.datetime(2024, 1, 2), "6h").alias("t")))
simple("fn_date_ranges", lambda: base.select("d").drop_nulls().head(2).select(pl.date_ranges(col("d"), col("d").dt.offset_by("2d")).alias("r")))
simple("fn_int_ranges", lambda: base.select("k").drop_nulls().select(pl.int_ranges(col("k")).alias("r"), pl.int_ranges(1, col("k") + 2, 2).alias("r2")))
simple("fn_date_datetime", lambda: base.select("k").drop_nulls().select(
    pl.date(2024, col("k"), 15).alias("d"), pl.datetime(2024, 1, col("k"), 12, col("k"), 30).alias("t")))
simple("fn_duration", lambda: base.select("k").drop_nulls().select(
    pl.duration(days=col("k"), hours=1).alias("d"), (pl.date(2024, 1, 1) + pl.duration(weeks=col("k"))).alias("shifted")))
simple("fn_from_epoch", lambda: pl.DataFrame({"s": [0, 1_700_000_000], "d": [0, 19_700]}).select(
    pl.from_epoch("s").alias("s"), pl.from_epoch("d", time_unit="d").alias("d"), pl.from_epoch(col("s") * 1000, time_unit="ms").alias("ms")))
simple("fn_arg_where", lambda: base.select(pl.arg_where(col("x") > 2)))
simple("fn_corr_cov", lambda: base.select(
    pl.corr("id", "y").alias("pearson"), pl.corr("id", "y", method="spearman").alias("spearman"),
    pl.cov("id", "x").alias("cov"), pl.cov("id", "x", ddof=0).alias("cov0")))
simple("fn_nth", lambda: base.select(pl.nth(0, 2)))
simple("fn_repeat_linear_space", lambda: pl.select(pl.repeat("a", 3).alias("r"), pl.linear_space(0, 1, 3).alias("ls")))
simple("fn_select", lambda: pl.select(lit(1).alias("one"), (lit(2) * 3).alias("six")))
simple("fn_stat_shorthands", lambda: base.select(
    pl.std("y").alias("std"), pl.var("y", ddof=0).alias("var"), pl.quantile("y", 0.5).alias("q"),
    pl.implode("g").alias("all"), pl.approx_n_unique("g").alias("au")))


# namespaces: str, dt, list, cat, arr, and bin additions
words = pl.DataFrame({"w": ["the quick fox", "Lazy DOG", None, "a.b*c", "café", '{"a": 1, "b": [1, 2]}']})
simple("str_many", lambda: words.select(
    col("w").str.contains_any(["fox", "dog"]).alias("any"),
    col("w").str.contains_any(["fox", "dog"], ascii_case_insensitive=True).alias("any_ci"),
    col("w").str.replace_many(["quick", "DOG"], "X").alias("rep"),
    col("w").str.replace_many({"the": "a", "fox": "cat"}).alias("rep_map"),
    col("w").str.extract_many(["o", "a"]).alias("ext"),
    col("w").str.find_many(["o", "a"]).alias("find"),
))
simple("str_extract_groups", lambda: pl.DataFrame({"u": ["id=7&n=ab", "id=12", None]}).select(
    col("u").str.extract_groups(r"id=(?<id>\d+)(?:&n=(\w+))?").alias("g")).unnest("g"))
simple("str_split_exact", lambda: pl.DataFrame({"s": ["a,b,c", "d", None]}).select(
    col("s").str.split_exact(",", 1).alias("e"), col("s").str.split_exact(",", 1, inclusive=True).alias("i")).unnest("e", "i", separator="_"))
simple("str_encode_decode", lambda: pl.DataFrame({"s": ["hi", "", None]}).select(
    col("s").str.encode("hex").alias("hex"), col("s").str.encode("base64").alias("b64"),
    col("s").str.encode("hex").str.decode("hex").cast(pl.String).alias("back"),
))
simple("str_to_decimal", lambda: pl.DataFrame({"s": ["1.25", "-3.5", None]}).select(col("s").str.to_decimal(scale=2).cast(pl.Float64)))
simple("str_normalize_escape", lambda: words.select(
    col("w").str.normalize("NFD").str.len_chars().alias("nfd"), col("w").str.escape_regex().alias("esc")))
simple("str_json", lambda: pl.DataFrame({"j": ['{"a": 1, "b": "x"}', '{"a": null}', None]}).select(
    col("j").str.json_decode(pl.Struct({"a": pl.Int64, "b": pl.String})).alias("d"),
    col("j").str.json_path_match("$.b").alias("jb"),
).unnest("d"))
days = pl.DataFrame({"d": [dt.date(2024, 1, 5), dt.date(2024, 2, 29), dt.date(1999, 12, 31), None]})
simple("dt_parts_extra", lambda: days.select(
    col("d").dt.millennium().alias("mil"), col("d").dt.century().alias("cen"), col("d").dt.days_in_month().alias("dim")))
simple("dt_business", lambda: days.select(
    col("d").dt.add_business_days(1).alias("next"),
    col("d").dt.add_business_days(-2, roll="forward").alias("back"),
    col("d").dt.add_business_days(1, holidays=[dt.date(2024, 1, 8)]).alias("hol"),
    col("d").dt.is_business_day().alias("is"),
    col("d").dt.add_business_days(1, week_mask=[True, True, True, True, True, True, False]).alias("six"),
))
simple("dt_combine_replace", lambda: days.select(
    col("d").dt.combine(dt.time(12, 30)).alias("c"),
    col("d").dt.replace(day=1).alias("first"),
    col("d").cast(pl.Datetime).dt.replace(hour=6, minute=15).alias("t"),
))
simple("dt_time_unit", lambda: base.select(
    col("t").dt.with_time_unit("ms").cast(pl.Int64).alias("ms"),
    (col("t") - col("t").min()).dt.total_nanoseconds().alias("ns"),
))
simple("list_more", lambda: base.select(
    col("l").list.any().alias("any"),
    col("l").list.eval(pl.element() > 2).list.all().alias("all"),
    col("l").list.filter(pl.element() > 2).alias("filt"),
    col("l").list.gather_every(2).alias("ge"),
    col("l").list.agg(pl.element().sum()).alias("agg"),
))
simple("list_to_array", lambda: pl.DataFrame({"l": [[1, 2], [3, 4]]}).select(
    col("l").list.to_array(2).alias("a")).select(col("a").arr.sum().alias("sum"), col("a").arr.max().alias("max"), col("a").arr.to_list().alias("back")))
arrs = pl.DataFrame({"a": [[1, 5, 2], [4, None, 4], None]}, schema={"a": pl.Array(pl.Int64, 3)})
simple("arr_aggs", lambda: arrs.select(
    col("a").arr.min().alias("min"), col("a").arr.max().alias("max"), col("a").arr.sum().alias("sum"),
    col("a").arr.mean().alias("mean"), col("a").arr.median().alias("median"), col("a").arr.std().alias("std"),
    col("a").arr.n_unique().alias("nu"), col("a").arr.arg_max().alias("am"), col("a").arr.len().alias("len"),
))
simple("arr_ops", lambda: arrs.select(
    col("a").arr.sort(descending=True).alias("sort"), col("a").arr.reverse().alias("rev"),
    col("a").arr.get(1).alias("get"), col("a").arr.first().alias("first"), col("a").arr.last().alias("last"),
    col("a").arr.contains(4).alias("has4"), col("a").arr.count_matches(4).alias("n4"),
    col("a").arr.shift(1).alias("shift"), col("a").arr.head(2).alias("head"), col("a").arr.tail(2).alias("tail"),
    col("a").arr.unique(maintain_order=True).alias("uniq"),
    col("a").cast(pl.Array(pl.String, 3)).arr.join("-").alias("join"),
    col("a").arr.eval(pl.element() * 2).alias("eval"),
))
simple("arr_explode_struct", lambda: arrs.head(2).select(col("a").arr.to_struct().alias("s")).unnest("s"))
cats = pl.DataFrame({"c": ["apple", "banana", "apple", None]}, schema={"c": pl.Categorical})
simple("cat_ns", lambda: cats.select(
    col("c").cat.len_bytes().alias("lb"), col("c").cat.len_chars().alias("lc"),
    col("c").cat.starts_with("ba").alias("sw"), col("c").cat.ends_with("le").alias("ew"),
    col("c").cat.slice(1, 3).alias("sl"),
))
simple("cat_categories", lambda: cats.select(col("c").cat.get_categories().sort()))
simple("bin_ns", lambda: pl.DataFrame({"s": ["hello", "", None]}).select(col("s").cast(pl.Binary).alias("b")).select(
    col("b").bin.contains(b"ell").alias("has"), col("b").bin.starts_with(b"he").alias("sw"),
    col("b").bin.ends_with(b"lo").alias("ew"), col("b").bin.size().alias("size"), col("b").bin.size("kb").alias("kb"),
    col("b").bin.encode("hex").alias("hex"), col("b").bin.encode("base64").alias("b64"),
    col("b").bin.encode("hex").str.decode("hex").bin.slice(1, 2).cast(pl.String).alias("slice"),
))

# Series methods that mirror Expr
simple("series_mirrors_expr", lambda: pl.DataFrame([
    ys.ewm_mean(alpha=0.5).alias("ewm"), ys.rolling_quantile(0.5, window_size=2).alias("rq"),
    xs.rle_id().alias("rle"), ys.degrees().alias("deg"), ys.cut([0, 2]).cast(pl.String).alias("cut"),
]))
simple("series_scalar_reductions", lambda: {
    "skew": ys.skew(), "kurtosis": ys.kurtosis(), "entropy": xs.drop_nulls().entropy(),
    "nan_max": ys.nan_max(), "approx": gs.approx_n_unique(), "index_of": xs.index_of(7),
    "search": xs.drop_nulls().sort().search_sorted(4),
})


# selectors
wide = pl.DataFrame(
    {
        "id": [1], "i8": [1], "u32": [1], "f32": [1.0], "price_f64": [1.0], "name": ["a"], "flag": [True],
        "day": [dt.date(2024, 1, 1)], "ts_us": [dt.datetime(2024, 1, 1)], "ts_ms_utc": [dt.datetime(2024, 1, 1)],
        "dur": [dt.timedelta(days=1)], "tm": [dt.time(1, 2)], "cat": ["x"], "lst": [[1, 2]], "lst_s": [["a"]],
        "arr": [[1, 2]], "st": [{"a": 1}], "bin": [b"x"], "dec": [1],
    },
    schema_overrides={
        "i8": pl.Int8, "u32": pl.UInt32, "f32": pl.Float32, "ts_ms_utc": pl.Datetime("ms", "UTC"), "cat": pl.Categorical,
        "arr": pl.Array(pl.Int64, 2), "dec": pl.Decimal(10, 2),
    },
)
simple("selectors", lambda: {k: ",".join(wide.select(v).columns) for k, v in {
    "numeric": cs.numeric(), "integer": cs.integer(), "signed": cs.signed_integer(), "unsigned": cs.unsigned_integer(),
    "float": cs.float(), "decimal": cs.decimal(), "temporal": cs.temporal(), "categorical": cs.categorical(),
    "nested": cs.nested(), "struct": cs.struct(), "string": cs.string(), "boolean": cs.boolean(), "binary": cs.binary(),
    "date": cs.date(), "time": cs.time(), "all": cs.all(), "first": cs.first(), "last": cs.last(),
    "list": cs.list(), "list_str": cs.list(cs.string()), "array": cs.array(), "array_w2": cs.array(width=2),
    "datetime": cs.datetime(), "datetime_ms": cs.datetime("ms"), "datetime_utc": cs.datetime(time_zone="UTC"),
    "datetime_naive": cs.datetime(time_zone=None), "duration": cs.duration(),
    "by_dtype": cs.by_dtype(pl.Int64, pl.String), "by_name": cs.by_name("name", "id"),
    "by_index": cs.by_index(0, -1), "starts_with": cs.starts_with("ts_", "lst"), "ends_with": cs.ends_with("_utc"),
    "contains": cs.contains("f"), "matches": cs.matches("^[a-c]"), "exclude": cs.exclude("id", cs.numeric()),
    "union": cs.string() | cs.boolean(), "intersect": cs.numeric() & cs.starts_with("i"),
    "difference": cs.numeric() - cs.by_name("id"), "complement": ~cs.temporal() & ~cs.nested(),
    "xor": cs.integer() ^ cs.starts_with("i", "f"),
}.items()})
simple("selectors_in_ops", lambda: base.select(cs.numeric() - cs.by_name("id")).with_columns((cs.float() * 2).name.suffix("_x2")))


# SQL
simple("sql_frame", lambda: base.sql("SELECT g, SUM(x) AS sx, COUNT(*) AS n FROM self WHERE x IS NOT NULL GROUP BY g ORDER BY g NULLS LAST"))
simple("sql_context_join", lambda: pl.SQLContext({"b": base, "r": right}).execute(
    "SELECT b.id, b.g, r.z FROM b LEFT JOIN r ON b.g = r.g ORDER BY b.id", eager=True))
simple("sql_expressions", lambda: base.sql(
    "SELECT id, CASE WHEN x > 2 THEN 'big' ELSE 'small' END AS size, UPPER(s) AS us, "
    "COALESCE(y, 0) AS y0, EXTRACT(year FROM d) AS yr, x * 2 + 1 AS x2 FROM self ORDER BY id"))
simple("sql_window_cte", lambda: base.sql(
    "WITH t AS (SELECT id, g, x FROM self WHERE g IS NOT NULL) "
    "SELECT id, g, ROW_NUMBER() OVER (PARTITION BY g ORDER BY id) AS rn, SUM(x) OVER (PARTITION BY g) AS gx FROM t ORDER BY id"))
simple("sql_table_name", lambda: base.select("id", "x").sql("SELECT MAX(x) AS mx FROM df", table_name="df"))
simple("sql_function", lambda: pl.sql("SELECT k, COUNT(*) AS n FROM long GROUP BY k ORDER BY k", eager=True))
simple("sql_error", lambda: base.sql("SELECT nope FROM self"))


# more frame and expression methods
simple("frame_to_dummies", lambda: base.select("id", "g", "k").to_dummies(["g", "k"]))
simple("frame_to_dummies_opts", lambda: base.select("g").to_dummies(separator=":", drop_first=True, drop_nulls=True))
simple("series_to_dummies", lambda: gs.to_dummies())
simple("frame_horizontal", lambda: pl.DataFrame([
    base.select("x", "y", "k").sum_horizontal(), base.select("x", "y", "k").mean_horizontal(ignore_nulls=False),
    base.select("x", "y", "k").min_horizontal(), base.select("x", "y", "k").max_horizontal(),
]))
simple("frame_clear", lambda: pl.concat([base.select("id", "g").clear(), base.select("id", "g").clear(2)]))
simple("frame_replace_column", lambda: base.select("id", "g").head(2).replace_column(1, pl.Series("z", [9, 8])))
simple("frame_column_index", lambda: {"k": base.get_column_index("k")})
simple("group_by_quantile", lambda: base.group_by("g", maintain_order=True).quantile(0.5, "linear").select("g", "x", "y"))
simple("min_max_by", lambda: base.select(col("g").min_by("x").alias("min"), col("g").max_by("y").alias("max")))
simple("first_last_non_null", lambda: base.select(col("x").shift(1).first(ignore_nulls=True).alias("f"), col("g").last(ignore_nulls=True).alias("l")))
simple("expr_item", lambda: base.select(col("x").max().item()))
simple("cumulative_eval", lambda: base.select(col("x").cumulative_eval(pl.element().first() - pl.element().last())))
simple("bitwise", lambda: pl.DataFrame({"a": [1, 6, 12]}).select(
    col("a").bitwise_count_ones().alias("ones"), col("a").cast(pl.UInt8).bitwise_leading_zeros().alias("lz"),
    col("a").bitwise_trailing_zeros().alias("tz"),
    col("a").bitwise_and().alias("and"), col("a").bitwise_or().alias("or"), col("a").bitwise_xor().alias("xor"),
))
simple("struct_with_fields", lambda: base.select(pl.struct("id", "x").alias("s")).select(
    col("s").struct.with_fields(pl.field("x").mul(10).alias("x10"), (pl.field("id") + pl.field("x")).alias("sum"))).unnest("s"))
simple("fn_business_day_count", lambda: days.select(pl.business_day_count(col("d"), col("d").dt.offset_by("10d")).alias("n")))
simple("fn_format", lambda: base.select(pl.format("{} has {}", "g", col("x")).alias("f")))
simple("fn_arg_sort_by", lambda: base.select(pl.arg_sort_by(["k", "id"], descending=[True, False], nulls_last=True)))
simple("fn_concat_arr", lambda: base.select(pl.concat_arr("id", "k").alias("a")).select(col("a").arr.sum()))

simple("group_by_iter", lambda: pl.DataFrame(
    [{"key": k[0], "k2": k[1], "n": g.height, "ids": g["id"].to_list()} for k, g in base.group_by("g", col("x") > 2, maintain_order=True)]))



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
