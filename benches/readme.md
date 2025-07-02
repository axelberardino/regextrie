# Bench

## Implementations

Different benches has been run, comparging two implemenation:
  * **Naive:** An array containing regex, input is match among them all
  * **RegexTrie:** A regex trie containg the same regex, but with optimized matching

## Results

### Finding the best match inside a random set of 10 entries

|               Test                |  Elapsed  |
|-----------------------------------|-----------|
| random_best_match/Naive/10        | 101.25 ns |
| random_best_match/RegexTrie/10    | 108.00 ns |

RegexTrie is <span style="color:red">+6.25%</span> slower

### Finding the best match inside a random set of 1000 entries

|               Test                |  Elapsed  |
|-----------------------------------|-----------|
| random_best_match/Naive/1000      | 3.4198 µs |
| random_best_match/RegexTrie/1000  | 136.87 ns |

RegexTrie is <span style="color:green">25x</span> faster

### Finding the best match inside a random set of 10000 entries

|               Test                |  Elapsed  |
|-----------------------------------|-----------|
| random_best_match/Naive/10000     | 27.176 µs |
| random_best_match/RegexTrie/10000 | 133.61 ns |

RegexTrie is <span style="color:green">203x</span> faster

### Finding the best match inside a small url set of 600 entries

|               Test                |  Elapsed  |
|-----------------------------------|-----------|
| small_assets_best_match/Naive     | 9.8780 µs |
| small_assets_best_match/RegexTrie | 1.0044 µs |

RegexTrie is <span style="color:green">9.8x</span> faster

### Finding the best match inside a big url set of 10 000 entries

|               Test                |  Elapsed  |
|-----------------------------------|-----------|
| big_assets_best_match/Naive       | 442.23 µs |
| big_assets_best_match/RegexTrie   | 940.31 ns |

RegexTrie is <span style="color:green">470x</span> faster

### Loading time of 10 000 regex from a raw input at once

|               Test                |  Elapsed  |
|-----------------------------------|-----------|
| loading_time/Naive                | 164.72 ms |
| loading_time/RegexTrie            | 616.86 ms |

RegexTrie is <span style="color:red">3.74x</span> slower to load entries

### Analysis

We're clearly seeing that except when they are very few entries (~10), the
RegexTrie is a magnitude order better. Even if the set is growing, the response
time doesn't increase greatly.
The only downside is the loading can be long because all the regex must be
precompiled, and inserting into a tree is not O(1), but O(log n).


