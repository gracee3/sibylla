# Shuffle fixture provenance

`fisher-yates-v1.json` was authored specifically for Sibylla on 2026-07-21. It
pins the byte-order, rejection-sampling, Fisher-Yates, and reversal behavior for
algorithm version 1 using an explicitly deterministic test-only source. The
seed is non-secret test data and must never be treated as production entropy.
