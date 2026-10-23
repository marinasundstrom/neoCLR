# Rejected paired host-root experiment — 2026-10-23

An owned async fiber resumes by admitting two independent strong roots. This
experiment combined the two heap-validation walks and slot reservations into one
atomic internal operation. It preserved exact-base/trailing-header validation and
the two root lifetimes, but added a private entry point. No gain was established:
one-worker throughput was 723–724 versus 720–736 requests/s; four-worker throughput
was 2,460–2,474 versus 2,484–2,614. CPU per response was also worse on average.
The implementation was reverted; no new root API is retained.

measurement.json records the paired forward/reverse trials and hashes. Both use
the same compact-point object, 32 clients, 1 MiB/16 lanes per owner and 256/1,024
paced GC, with two seconds warmup/six seconds measurement. These are local short
trials, not proof that batching can never help. The native experiment is preserved
in rejected.patch. The draft dedicated test was not run, and this experiment is
not qualified for use. measure.py can repeat the comparison after applying the
patch, using baseline f075e8a96 and target/plain-compact-metadata/candidate.o.

No weakening of heap validation, allocation placement or root ownership was
accepted. The follow-up targets remaining storage/transient diagnostic metadata.
